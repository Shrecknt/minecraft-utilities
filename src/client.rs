use std::error::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::packetutil::{get_packet, send_prefixed_packet, MinecraftPacket};

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

    pub async fn join(&mut self) -> Result<MinecraftPacket, Box<dyn Error>> {
        match &mut self.connection {
            Some(stream) => {
                let protocol_version: usize = 762; // 0xf805;
                let hostname = "shrecked.dev";
                let port = 25565;
                let playername = "Shrecknt";

                let mut connect_packet: Vec<u8> = vec![];
                connect_packet.write_u8(0x00).await?;
                varint_rs::VarintWriter::write_usize_varint(&mut connect_packet, protocol_version)?; // protocol version - 760 (1.19.2)
                varint_rs::VarintWriter::write_usize_varint(
                    &mut connect_packet,
                    hostname.as_bytes().len(),
                )?; // host length - 12
                connect_packet.write_all(hostname.as_bytes()).await?; // host name - shrecked.dev
                connect_packet.write_u16(port).await?; // port number - 42069
                connect_packet.write_u8(0x02).await?; // next state - 2 (login)

                send_prefixed_packet(stream, &connect_packet).await?;

                let mut login_start_packet: Vec<u8> = vec![];
                login_start_packet.write_u8(0x00).await?;
                varint_rs::VarintWriter::write_usize_varint(
                    &mut login_start_packet,
                    playername.as_bytes().len(),
                )?;
                login_start_packet.write_all(playername.as_bytes()).await?;
                login_start_packet.write_u8(0x01).await?;
                let uuid = Uuid::new_v4();
                login_start_packet.write_all(uuid.as_bytes()).await?;

                send_prefixed_packet(stream, &login_start_packet).await?;

                println!("{:?}", stream);

                let result = get_packet(stream).await?;
                Ok(result)
            }
            None => {
                Err("No connection, cannot join".into())
            }
        }
    }

    pub async fn check_online_mode(&mut self) -> Result<bool, Box<dyn Error>> {
        let res = self.join().await?;
        Ok(res.packet_id == 0x01)
    }
}
