use serde_json::Value;
use std::{error::Error, vec};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::packetutil::{read_varint, send_prefixed_packet};

#[derive(Debug)]
pub struct Ping {}

impl Ping {
    pub async fn ping(
        addr: &str,
        input_protocol_version: Option<usize>,
        input_hostname: Option<&str>,
        input_port: Option<u16>,
    ) -> Result<Value, Box<dyn Error>> {
        const DEFAULT_PROTOCOL_VERSION: usize = 0xf807;
        const DEFAULT_HOSTNAME: &str = "shrecked.dev";
        const DEFAULT_PORT: u16 = 25565;

        let protocol_version = input_protocol_version.unwrap_or(DEFAULT_PROTOCOL_VERSION);
        let hostname = input_hostname.unwrap_or(DEFAULT_HOSTNAME);
        let port = input_port.unwrap_or(DEFAULT_PORT);

        let mut connection = TcpStream::connect(addr).await?;

        let mut connect_packet: Vec<u8> = vec![];
        connect_packet.write_u8(0x00).await?;
        varint_rs::VarintWriter::write_usize_varint(&mut connect_packet, protocol_version)?; // protocol version - 762 (1.19.4)
        varint_rs::VarintWriter::write_usize_varint(&mut connect_packet, hostname.len())?; // host length - 12
        connect_packet.write_all(hostname.as_bytes()).await?; // host name - shrecked.dev
        connect_packet.write_u16(port).await?; // port number - 42069
        connect_packet.write_u8(0x01).await?; // next state - 1 (ping)

        send_prefixed_packet(&mut connection, &connect_packet).await?;

        let mut ping_packet: Vec<u8> = vec![];
        ping_packet.write_u8(0x00).await?;

        send_prefixed_packet(&mut connection, &ping_packet).await?;

        connection.readable().await?;

        let _debug_0 = read_varint(&mut connection).await?;
        let _debug_1 = connection.read_u8().await?;
        if _debug_1 != 0x00 {
            return Err("Invalid response".into());
        }
        let len = read_varint(&mut connection).await?;

        let mut data = vec![0; len.try_into().unwrap()];
        connection.read_exact(&mut data).await?;

        let source: String = data.iter().map(|x| char::from(*x)).collect();
        let parse_res = serde_json::from_str(&source);

        match parse_res {
            Ok(v) => Ok(v),
            Err(err) => Err(Box::new(err)),
        }
    }

    pub fn get_protocol_version(json: Value) -> Result<i32, Box<dyn Error>> {
        let val = json["version"]["protocol"].clone().as_i64();
        match val {
            Some(res) => Ok(res.try_into().unwrap()),
            None => Err("Could not find protocol version".into()),
        }
    }
}
