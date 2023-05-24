use serde_json::Value;
use std::{error::Error, vec};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::packetutil::{read_varint, send_prefixed_packet, write_varint};

#[derive(Debug)]
pub struct LegacyPingResult {
    pub protocol_version: u8,
    pub server_version: String,
    pub motd: String,
    pub player_count: isize,
    pub max_player_count: isize,
}

impl LegacyPingResult {
    pub fn default() -> Self {
        LegacyPingResult {
            protocol_version: 0,
            server_version: String::from(""),
            motd: String::from(""),
            player_count: -1,
            max_player_count: -1,
        }
    }
}

#[derive(Debug)]
pub struct Ping {}

impl Ping {
    pub async fn ping(
        host: &str,
        port: Option<u16>,
        input_protocol_version: Option<usize>,
        input_hostname: Option<&str>,
        input_port: Option<u16>,
    ) -> Result<Value, Box<dyn Error>> {
        const DEFAULT_PROTOCOL_VERSION: usize = 0xf807;
        const DEFAULT_HOSTNAME: &str = "shrecked.dev";
        const DEFAULT_PORT: u16 = 25565;

        let send_protocol_version = input_protocol_version.unwrap_or(DEFAULT_PROTOCOL_VERSION);
        let send_hostname = input_hostname.unwrap_or(DEFAULT_HOSTNAME);
        let send_port = input_port.unwrap_or(DEFAULT_PORT);

        let mut connection =
            TcpStream::connect(format!("{}:{}", host, port.unwrap_or(25565))).await?;

        let mut connect_packet: Vec<u8> = vec![];
        connect_packet.write_u8(0x00).await?;
        write_varint(&mut connect_packet, i32::try_from(send_protocol_version)?).await?; // protocol version - 762 (1.19.4)
        write_varint(&mut connect_packet, i32::try_from(send_hostname.len())?).await?; // host length - 12
        connect_packet.write_all(send_hostname.as_bytes()).await?; // host name - shrecked.dev
        connect_packet.write_u16(send_port).await?; // port number - 42069
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

        let mut data = vec![0; len.try_into()?];
        connection.read_exact(&mut data).await?;

        let source: String = data.iter().map(|x| char::from(*x)).collect();
        let parse_res: Result<Value, serde_json::Error> = serde_json::from_str(&source);

        Ok(parse_res?)
    }

    pub fn get_protocol_version(json: &Value) -> Result<i32, Box<dyn Error>> {
        let val = json["version"]["protocol"].clone().as_i64();
        match val {
            Some(res) => Ok(res.try_into()?),
            None => Err("Could not find protocol version".into()),
        }
    }

    pub async fn ping_legacy_protocol(
        host: &str,
        port: Option<u16>,
        input_protocol_version: Option<u8>,
        input_hostname: Option<&str>,
        input_port: Option<u16>,
    ) -> Result<LegacyPingResult, Box<dyn Error>> {
        const DEFAULT_PROTOCOL_VERSION: u8 = 69;
        const DEFAULT_HOSTNAME: &str = "shrecked.dev";
        const DEFAULT_PORT: u16 = 25565;

        let send_protocol_version = input_protocol_version.unwrap_or(DEFAULT_PROTOCOL_VERSION);
        let send_hostname = input_hostname.unwrap_or(DEFAULT_HOSTNAME);
        let send_port = input_port.unwrap_or(DEFAULT_PORT);

        let mut connection =
            TcpStream::connect(format!("{}:{}", host, port.unwrap_or(25565))).await?;

        let mut ping_packet: Vec<u8> = vec![];
        ping_packet
            .write_all(&[
                0xFEu8, 0x01, 0xFA, 0x00, 0x0B, 0x00, 0x4D, 0x00, 0x43, 0x00, 0x7C, 0x00, 0x50,
                0x00, 0x69, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x48, 0x00, 0x6F, 0x00, 0x73, 0x00, 0x74,
            ])
            .await?;
        let write_hostname = send_hostname.encode_utf16().collect::<Vec<u16>>();
        let mut write_hostname_bytes: Vec<u8> = vec![];
        for i in write_hostname.chunks_exact(1) {
            let bytes: [u8; 2] = i[0].to_be_bytes();
            write_hostname_bytes.push(bytes[0]);
            write_hostname_bytes.push(bytes[1]);
        }
        ping_packet
            .write_u16(u16::try_from(7 + write_hostname_bytes.len())?)
            .await?;
        ping_packet.write_u8(send_protocol_version).await?;
        ping_packet
            .write_u16(u16::try_from(write_hostname.len())?)
            .await?;
        ping_packet.write_all(&write_hostname_bytes).await?;
        ping_packet.write_i32(i32::from(send_port)).await?;

        connection.write_all(&ping_packet).await?;

        let identifier = connection.read_u8().await?;
        if identifier != 0xFF {
            return Err(format!("Bad identifier '{identifier}'").into());
        }

        let _len = connection.read_u16().await?;
        let mut res = vec![];
        connection.read_to_end(&mut res).await?;
        let mut res_u16s: Vec<u16> = vec![];
        for chunk in res.chunks_exact(2) {
            res_u16s.push(((chunk[0] as u16) << 8) | chunk[1] as u16);
        }
        let res_string = char::decode_utf16(res_u16s)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>();
        let mut res_iter = res_string.split('\0');

        let _header = res_iter.next();
        let mut ping_res = LegacyPingResult::default();
        ping_res.protocol_version = str::parse::<u8>(res_iter.next().unwrap_or("0"))?;
        ping_res.server_version = res_iter.next().unwrap_or("???").to_string();
        ping_res.motd = res_iter.next().unwrap_or("???").to_string();
        ping_res.player_count = str::parse::<isize>(res_iter.next().unwrap_or("-1"))?;
        ping_res.max_player_count = str::parse::<isize>(res_iter.next().unwrap_or("-1"))?;

        Ok(ping_res)
    }
}
