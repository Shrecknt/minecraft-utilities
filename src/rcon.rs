use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct RconClient {
    request_id: i32,
    connected: bool,
    stream: Option<TcpStream>,
}

impl RconClient {
    pub async fn connect(addr: &str, password: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let mut client = RconClient {
            request_id: 0,
            connected: false,
            stream: None,
        };

        let connection = TcpStream::connect(addr).await?;

        client.stream = Some(connection);
        client.connected = true;

        if password.is_some() {
            client.login(password.unwrap()).await?;
        } else {
            client.connected = false;
        }

        Ok(client)
    }

    pub async fn login(&mut self, password: &str) -> Result<RconPacket, Box<dyn Error>> {
        if self.connected {
            return Err("Attempted to authenticate while already logged into RCON server".into());
        }

        let mut login_packet = RconPacket::new();
        let login_packet_id = self.request_id;
        login_packet.request_id = self.request_id;
        self.request_id += 1;
        login_packet.request_type = 3;
        login_packet.payload = password.into();

        let login_res = self.send_packet(login_packet).await?;

        if login_res.request_id == -1 {
            self.connected = false;
            return Err(
                "Authentication failure, make sure you typed your password correctly".into(),
            );
        } else if login_res.request_id != login_packet_id {
            self.connected = false;
            return Err(
                "Recieved a strange packet from server. Are you sure this is an RCON server?"
                    .into(),
            );
        }

        self.connected = true;

        return Ok(login_res);
    }

    async fn send_packet(&mut self, packet: RconPacket) -> Result<RconPacket, Box<dyn Error>> {
        match &mut self.stream {
            Some(stream) => {
                let built_packet = packet.build().await?;
                stream.write_all(&built_packet).await?;

                stream.readable().await?;
                let size = stream.read_i32_le().await?;
                let mut res_buf: Vec<u8> = vec![0; size as usize];
                let read_res = stream.read_exact(&mut res_buf).await;
                if read_res.is_err() {
                    return Err("The server lied".into());
                }
                let mut res_return_packet = RconPacket::new();
                res_return_packet.parse(&res_buf).await?;

                return Ok(res_return_packet);
            }
            None => {
                return Err(
                    "Attempted to send packet before establishing a connection to server".into(),
                );
            }
        }
    }

    pub async fn command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        if !self.connected {
            return Err("Attempted to send command before client is authenticated".into());
        }

        let mut packet = RconPacket::new();
        packet.request_id = self.request_id;
        self.request_id += 1;
        packet.request_type = 2;
        packet.payload = command.into();

        let res = self.send_packet(packet).await?;

        Ok(res.payload_to_string())
    }

    pub fn to_string(&self) -> String {
        format!(
            "Request ID: {}, Connected: {}, Stream: {:?}",
            self.request_id, self.connected, self.stream
        )
    }
}

pub struct RconPacket {
    pub request_id: i32,
    pub request_type: i32,
    pub payload: Vec<u8>,
}

impl RconPacket {
    pub fn new() -> Self {
        RconPacket {
            request_id: 0,
            request_type: 0,
            payload: vec![],
        }
    }

    pub async fn build(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut packet: Vec<u8> = vec![];
        packet.write_i32_le(self.request_id as i32).await?;
        packet.write_i32_le(self.request_type as i32).await?; // 3 for login, 2 to run a command, 0 for a multi-packet response
        packet.write(&self.payload).await?;
        packet.write(b"\0\0").await?;
        let mut buffer: Vec<u8> = vec![];
        buffer.write_i32_le(packet.len() as i32).await?;
        buffer.write(&packet).await?;

        Ok(buffer)
    }

    pub async fn parse(&mut self, raw: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let request_id = as_i32_le(&raw[0..4].try_into().unwrap());
        let request_type = as_i32_le(&raw[4..8].try_into().unwrap());
        let mut payload = (&raw[8..]).to_vec();
        payload.pop();
        payload.pop();

        self.request_id = request_id;
        self.request_type = request_type;
        self.payload = payload;

        Ok(())
    }

    pub fn payload_to_string(&self) -> String {
        self.payload.iter().map(|x| char::from(*x)).collect()
    }

    pub fn to_string(&self) -> String {
        let str = self
            .payload_to_string()
            .replace("\\", "\\\\")
            .replace("\"", "\\\"");
        format!(
            "{{ \"request_id\": \"{}\", \"request_type\": \"{}\", \"payload\": \"{}\" }}",
            self.request_id, self.request_type, str
        )
    }
}

impl std::fmt::Display for RconClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for RconPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn as_i32_le(array: &[u8; 4]) -> i32 {
    ((array[0] as i32) << 0)
        + ((array[1] as i32) << 8)
        + ((array[2] as i32) << 16)
        + ((array[3] as i32) << 24)
}
