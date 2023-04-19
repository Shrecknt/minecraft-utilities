use std::{error::Error, vec};
use json::JsonValue;
use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};


pub struct Ping {
    pub contents: Option<JsonValue>
}

impl Ping {
    pub async fn ping(addr: &str) -> Result<Self, Box<dyn Error>> {
        let mut val = Ping {
            contents: None
        };

        let mut connection = TcpStream::connect(addr).await?;

        let mut connect_packet: Vec<u8> = vec![];
        connect_packet.write_u8(0x00).await?;
        connect_packet.write_u16(0xf805).await?; // protocol version - 760 (1.19.2)
        connect_packet.write_u8(0x0c).await?; // host length - 12
        connect_packet.write("shrecked.dev".as_bytes()).await?; // host name - shrecked.dev
        connect_packet.write_u16(42069).await?; // port number - 42069
        connect_packet.write_u8(0x01).await?; // next state - 1 (ping)

        println!("Sending: {:?}", connect_packet);

        send_prefixed_packet(&mut connection, &connect_packet).await?;

        let mut ping_packet: Vec<u8> = vec![];
        ping_packet.write_u8(0x00).await?;

        println!("Sending: {:?}", ping_packet);

        send_prefixed_packet(&mut connection, &ping_packet).await?;

        connection.readable().await?;
        let parse_res = val.generate_contents(&mut connection).await;
        match parse_res {
            Ok(_) => {
                return Ok(val);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    async fn generate_contents(&mut self, connection: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let mut len = read_varint(connection).await?;
        connection.read_u8().await?;
        len -= 1;

        let mut data = vec![0; len as usize];
        connection.read_exact(&mut data).await?;
        println!("Got len: {}, data: {:?}", len, data);

        let source: String = data.iter().map(|x| char::from(*x)).collect();
        let res = json::parse(&source);
        match res {
            Ok(contents) => {
                self.contents = Some(contents);
                Ok(())
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
}

async fn send_prefixed_packet(connection: &mut TcpStream, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    varint_rs::VarintWriter::write_usize_varint(&mut buffer, data.len())?;
    buffer.write(&data).await?;

    connection.write_all(&buffer).await?;

    Ok(())
}

async fn read_varint(stream: &mut TcpStream) -> Result<i32, Box<dyn Error>> {
    const LAST_SEVEN_BITS: i32 = 0b0111_1111;
    const NEXT_BYTE_EXISTS: u8 = 0b1000_0000;

    let mut buffer = [0u8];
    let mut result = 0;
    let mut read_count = 0u32;
    loop {
        stream.read_exact(&mut buffer).await?;
        result |= (buffer[0] as i32 & LAST_SEVEN_BITS).checked_shl(7 * read_count)
            .ok_or("Unsupported protocol")?;

        read_count += 1;
        if read_count > 5 {
            break Err("Unsupported protocol".into());
        } else if (buffer[0] & NEXT_BYTE_EXISTS) == 0 {
            break Ok(result);
        }
    }
}
