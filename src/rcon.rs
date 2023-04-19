use std::error::Error;
use tokio::{io::{AsyncWriteExt, AsyncReadExt}, net::TcpStream};

pub struct RconClient {
    request_id: i32,
    connected: bool,
    stream: Option<TcpStream>
}

impl RconClient {
    pub async fn connect(addr: &str, password: &str) -> Result<Self, Box<dyn Error>> {
        let mut client = RconClient {
            request_id: 0,
            connected: false,
            stream: None
        };

        let connection = TcpStream::connect(addr).await?;

        client.stream = Some(connection);
        client.connected = true;

        let mut login_packet = RconPacket::new();
        login_packet.request_id = client.request_id;
        client.request_id += 1;
        login_packet.request_type = 3;
        login_packet.payload = password.into();

        let login_res = client.send_packet(login_packet).await?;

        if login_res.request_id == -1 {
            client.connected = false;
            client.stream = None;
            panic!("Incorrect password");
        } else if login_res.request_id != 0 {
            panic!("are you sure you connected to an rcon server?");
        }

        Ok(client)
    }

    async fn send_packet(&mut self, packet: RconPacket) -> Result<RconPacket, Box<dyn Error>> {
        if !&self.connected {
            panic!("some error message");
        }
        match &mut self.stream {
            Some(stream) => {
                let built_packet = packet.build().await?;
                stream.write_all(&built_packet).await?;

                let mut res_buf: Vec<u8> = vec![];
                stream.readable().await?;
                let _size = stream.read_buf(&mut res_buf).await?;
                let mut res_return_packet = RconPacket::new();
                res_return_packet.parse(&res_buf).await?;

                return Ok(res_return_packet);
            }
            None => {
                panic!("How did you get here??");
            }
        }
    }

    pub async fn command(&mut self, command: &str) -> Result<String, Box<dyn Error>>{
        let mut packet = RconPacket::new();
        packet.request_id = self.request_id;
        self.request_id += 1;
        packet.request_type = 2;
        packet.payload = command.into();

        let res = self.send_packet(packet).await?;

        Ok(res.payload_to_string())
    }
}

pub struct RconPacket {
    pub request_id: i32,
    pub request_type: i32,
    pub payload: Vec<u8>
}

impl RconPacket {
    pub fn new() -> Self {
        RconPacket {
            request_id: 0,
            request_type: 0,
            payload: vec![]
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
        let _length = as_i32_le(&raw[0..4].try_into().unwrap());
        //println!("Packet Length: {}", _length);
        let request_id = as_i32_le(&raw[4..8].try_into().unwrap());
        let request_type = as_i32_le(&raw[8..12].try_into().unwrap());
        let mut payload = (&raw[13..]).to_vec();
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

    pub fn print(&self) {
        let str = self.payload_to_string();
        println!("Request ID: {}, Request Type: {}, payload: {}", self.request_id, self.request_type, str);
    }
}

fn as_i32_le(array: &[u8; 4]) -> i32 {
    ((array[0] as i32) <<  0) +
    ((array[1] as i32) <<  8) +
    ((array[2] as i32) << 16) +
    ((array[3] as i32) << 24)
}