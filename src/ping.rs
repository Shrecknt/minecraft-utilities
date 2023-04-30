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

        //println!("Sending: {:?}", connect_packet);

        send_prefixed_packet(&mut connection, &connect_packet).await?;

        let mut ping_packet: Vec<u8> = vec![];
        ping_packet.write_u8(0x00).await?;

        //println!("Sending: {:?}", ping_packet);

        send_prefixed_packet(&mut connection, &ping_packet).await?;

        connection.readable().await?;
        let parse_res = Ping::generate_contents(&mut connection).await;
        match parse_res {
            Ok(v) => Ok(v),
            Err(err) => Err(err),
        }
    }

    async fn generate_contents(connection: &mut TcpStream) -> Result<Value, Box<dyn Error>> {
        //let mut debug_buf: Vec<u8> = vec![];
        //connection.read_buf(&mut debug_buf).await?;
        //println!("debug_buf: {:?}", debug_buf);

        let _debug_0 = read_varint(connection).await?;
        // total packet length - ignore
        //println!("_debug_0: {}", _debug_0);

        let _debug_1 = connection.read_u8().await?;
        assert_eq!(_debug_1, 0x00);
        // packet id - 0x00
        //println!("_debug_1: {}", _debug_1);

        let len = read_varint(connection).await?;
        // length of remaining packet
        //println!("len: {}", len);

        let mut data = vec![0; len.try_into().unwrap()];
        connection.read_exact(&mut data).await?;

        let source: String = data.iter().map(|x| char::from(*x)).collect();
        let res: Result<Value, serde_json::Error> = serde_json::from_str(&source);
        match res {
            Ok(contents) => Ok(contents),
            Err(err) => Err(err.into()),
        }
    }
}
