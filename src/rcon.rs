use std::error::Error;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::ServerAddress;

#[derive(Error, Debug)]
pub enum RconError {
    #[error("Attempted to authenticate while already logged into RCON server.")]
    DoubleLogin,
    #[error("Authentication failure, make sure you sent the correct password.")]
    AuthenticationFailure,
    #[error("Recieved a strange packet from the server. Are you sure this is an RCON server?")]
    StrangePacket,
    #[error("Attempted to send packet before establishing a connection to the server.")]
    EarlyPacket,
    #[error("Attempted to send command before client is authenticated.")]
    EarlyCommand,
}

#[derive(Debug)]
pub struct RconClient {
    request_id: i32,
    connected: bool,
    stream: Option<TcpStream>,
}

impl RconClient {
    pub async fn connect(
        addr: &ServerAddress,
        password: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut client = RconClient {
            request_id: 0,
            connected: false,
            stream: None,
        };

        let connection = TcpStream::connect(format!("{}:{}", addr.host, addr.port)).await?;

        client.stream = Some(connection);
        client.connected = true;

        match password {
            Some(password) => {
                client.login(password).await?;
            }
            None => {
                client.connected = false;
            }
        }

        Ok(client)
    }

    pub async fn login(&mut self, password: &str) -> Result<RconPacket, Box<dyn Error>> {
        if self.connected {
            return Err(RconError::DoubleLogin.into());
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
            return Err(RconError::AuthenticationFailure.into());
        } else if login_res.request_id != login_packet_id {
            self.connected = false;
            return Err(RconError::StrangePacket.into());
        }

        self.connected = true;

        Ok(login_res)
    }

    async fn send_packet(&mut self, packet: RconPacket) -> Result<RconPacket, Box<dyn Error>> {
        match &mut self.stream {
            Some(stream) => {
                let built_packet = packet.build().await?;
                stream.write_all(&built_packet).await?;

                stream.readable().await?;
                let size = stream.read_i32_le().await?;
                let mut res_buf: Vec<u8> = vec![0; size as usize];
                stream.read_exact(&mut res_buf).await?;
                let mut res_return_packet = RconPacket::new();
                res_return_packet.parse(&res_buf).await?;

                Ok(res_return_packet)
            }
            None => Err(RconError::EarlyPacket.into()),
        }
    }

    pub async fn command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        if !self.connected {
            return Err(RconError::EarlyCommand.into());
        }

        let mut packet = RconPacket::new();
        packet.request_id = self.request_id;
        self.request_id += 1;
        packet.request_type = 2;
        packet.payload = command.into();

        let res = self.send_packet(packet).await?;

        Ok(res.payload_to_string())
    }
}

#[derive(Debug)]
pub struct RconPacket {
    pub request_id: i32,
    pub request_type: i32,
    pub payload: Vec<u8>,
}

impl Default for RconPacket {
    fn default() -> Self {
        Self::new()
    }
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
        packet.write_i32_le(self.request_id).await?;
        packet.write_i32_le(self.request_type).await?; // 3 for login, 2 to run a command, 0 for a multi-packet response
        packet.write_all(&self.payload).await?;
        packet.write_all(b"\0\0").await?;
        let mut buffer: Vec<u8> = vec![];
        buffer.write_i32_le(packet.len() as i32).await?;
        buffer.write_all(&packet).await?;

        Ok(buffer)
    }

    pub async fn parse(&mut self, raw: &[u8]) -> Result<(), Box<dyn Error>> {
        let request_id = as_i32_le(&raw[0..4].try_into()?);
        let request_type = as_i32_le(&raw[4..8].try_into()?);
        let mut payload = (raw[8..]).to_vec();
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
}

fn as_i32_le(array: &[u8; 4]) -> i32 {
    (array[0] as i32)
        + ((array[1] as i32) << 8)
        + ((array[2] as i32) << 16)
        + ((array[3] as i32) << 24)
}
