use std::error::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::packetutil::{get_packet, send_prefixed_packet, MinecraftPacket};

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
    pub async fn connect(host: String, port: Option<u16>) -> Result<Self, Box<dyn Error>> {
        let mut res = Client {
            host,
            port: port.unwrap_or(25565),
            connection: None,
        };

        res.connection = Some(TcpStream::connect(format!("{}:{}", res.host, res.port)).await?);

        Ok(res)
    }

    pub async fn join(
        &mut self,
        protocol_version: Option<usize>,
        hostname: Option<&str>,
        port: Option<u16>,
        playername: Option<&str>,
    ) -> Result<MinecraftPacket, Box<dyn Error>> {
        match &mut self.connection {
            Some(stream) => {
                let resolved_protocol_version: usize = protocol_version.unwrap_or(762);
                let resolved_hostname = hostname.unwrap_or("shrecked.dev");
                let resolved_port = port.unwrap_or(25565);
                let resolved_playername = playername.unwrap_or("Shrecknt");

                let mut connect_packet: Vec<u8> = vec![];
                connect_packet.write_u8(0x00).await?;
                varint_rs::VarintWriter::write_usize_varint(
                    &mut connect_packet,
                    resolved_protocol_version,
                )?; // protocol version - 760 (1.19.2)
                varint_rs::VarintWriter::write_usize_varint(
                    &mut connect_packet,
                    resolved_hostname.as_bytes().len(),
                )?; // host length - 12
                connect_packet
                    .write_all(resolved_hostname.as_bytes())
                    .await?; // host name - shrecked.dev
                connect_packet.write_u16(resolved_port).await?; // port number - 42069
                connect_packet.write_u8(0x02).await?; // next state - 2 (login)

                send_prefixed_packet(stream, &connect_packet).await?;

                let mut login_start_packet: Vec<u8> = vec![];
                login_start_packet.write_u8(0x00).await?;
                varint_rs::VarintWriter::write_usize_varint(
                    &mut login_start_packet,
                    resolved_playername.as_bytes().len(),
                )?;
                login_start_packet
                    .write_all(resolved_playername.as_bytes())
                    .await?;
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
        protocol_version: Option<usize>,
        hostname: Option<&str>,
        port: Option<u16>,
        playername: Option<&str>,
    ) -> Result<OnlineModeResults, Box<dyn Error>> {
        let res = self
            .join(protocol_version, hostname, port, playername)
            .await?;
        println!("packet_id: {}", res.packet_id);
        if res.packet_id == 0x00 {
            Ok(OnlineModeResults::Kicked)
        } else if res.packet_id == 0x01 {
            Ok(OnlineModeResults::OnlineMode)
        } else if res.packet_id == 0x03 {
            Ok(OnlineModeResults::OfflineMode)
        } else {
            Ok(OnlineModeResults::UnknownProtocol)
        }
    }
}
