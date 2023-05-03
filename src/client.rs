use std::{error::Error, str::from_utf8};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::packetutil::{
    get_packet, read_varint_buf, send_prefixed_packet, write_varint, MinecraftPacket,
};

#[derive(Debug)]
pub enum OnlineModeResults {
    OnlineMode,
    OfflineMode,
    Kicked,
    UnknownProtocol,
}

#[derive(Debug)]
pub struct Client {
    host: String,
    port: u16,
    connection: Option<TcpStream>,
}

impl Client {
    pub async fn connect(host: &str, port: Option<u16>) -> Result<Self, Box<dyn Error>> {
        let mut res = Client {
            host: String::from(host),
            port: port.unwrap_or(25565),
            connection: None,
        };

        res.connection = Some(TcpStream::connect(format!("{}:{}", res.host, res.port)).await?);

        Ok(res)
    }

    pub async fn join(
        &mut self,
        protocol_version: Option<i32>,
        hostname: Option<&str>,
        port: Option<u16>,
        playername: Option<&str>,
    ) -> Result<MinecraftPacket, Box<dyn Error>> {
        match &mut self.connection {
            Some(stream) => {
                let resolved_protocol_version: i32 = protocol_version.unwrap_or(762);
                let resolved_hostname = hostname.unwrap_or("shrecked.dev");
                let resolved_port = port.unwrap_or(25565);
                let resolved_playername = playername.unwrap_or("Shrecknt");

                let mut connect_packet: Vec<u8> = vec![];
                connect_packet.write_u8(0x00).await?;
                write_varint(&mut connect_packet, resolved_protocol_version).await?; // protocol version - 762 (1.19.4)
                write_varint(
                    &mut connect_packet,
                    i32::try_from(resolved_hostname.as_bytes().len())?,
                )
                .await?; // host length - 12
                connect_packet
                    .write_all(resolved_hostname.as_bytes())
                    .await?; // host name - shrecked.dev
                connect_packet.write_u16(resolved_port).await?; // port number - 42069
                connect_packet.write_u8(0x02).await?; // next state - 2 (login)

                send_prefixed_packet(stream, &connect_packet).await?;

                let mut login_start_packet: Vec<u8> = vec![];
                login_start_packet.write_u8(0x00).await?;
                write_varint(
                    &mut login_start_packet,
                    i32::try_from(resolved_playername.as_bytes().len())?,
                )
                .await?;
                login_start_packet
                    .write_all(resolved_playername.as_bytes())
                    .await?;
                if resolved_protocol_version == 759 || resolved_protocol_version == 760 {
                    login_start_packet.write_u8(0x00).await?;
                }
                login_start_packet.write_u8(0x01).await?;
                let uuid = Uuid::new_v4();
                login_start_packet.write_all(uuid.as_bytes()).await?;

                send_prefixed_packet(stream, &login_start_packet).await?;

                let result = get_packet(stream).await?;
                Ok(result)
            }
            None => Err("No connection, cannot join".into()),
        }
    }

    pub async fn check_online_mode(
        &mut self,
        protocol_version: Option<i32>,
        hostname: Option<&str>,
        port: Option<u16>,
        playername: Option<&str>,
    ) -> Result<(OnlineModeResults, Option<String>), Box<dyn Error>> {
        let res = self
            .join(protocol_version, hostname, port, playername)
            .await?;
        if res.packet_id == 0x00 {
            let (len, val) = read_varint_buf(&res.buffer).await?;
            if val > 16777216 {
                return Err("Exceeded varint size sanity limit".into());
            }
            let slice = &res.buffer[len.try_into()?..(u32::try_from(val)? + len).try_into()?];
            let reason = from_utf8(slice)?;
            Ok((OnlineModeResults::Kicked, Some(reason.to_string())))
        } else if res.packet_id == 0x01 {
            Ok((OnlineModeResults::OnlineMode, None))
        } else if res.packet_id == 0x03 {
            Ok((OnlineModeResults::OfflineMode, None))
        } else {
            Ok((OnlineModeResults::UnknownProtocol, None))
        }
    }
}
