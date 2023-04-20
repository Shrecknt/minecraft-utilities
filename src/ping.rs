use json::JsonValue;
use std::{error::Error, vec};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Ping {
    pub contents: Option<JsonValue>,
}

impl Ping {
    pub async fn ping(
        addr: &str,
        input_protocol_version: Option<u16>,
        input_hostname: Option<&str>,
        input_port: Option<u16>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut val = Ping { contents: None };

        const DEFAULT_PROTOCOL_VERSION: u16 = 0xf805;
        const DEFAULT_HOSTNAME: &str = "shrecked.dev";
        const DEFAULT_PORT: u16 = 25565;

        let protocol_version = input_protocol_version.unwrap_or(DEFAULT_PROTOCOL_VERSION);
        let hostname = input_hostname.unwrap_or(DEFAULT_HOSTNAME);
        let port = input_port.unwrap_or(DEFAULT_PORT);

        let mut connection = TcpStream::connect(addr).await?;

        let mut connect_packet: Vec<u8> = vec![];
        connect_packet.write_u8(0x00).await?;
        connect_packet.write_u16(protocol_version).await?; // protocol version - 760 (1.19.2)
                                                           //connect_packet.write_u8(0x0c).await?; // host length - 12
        varint_rs::VarintWriter::write_usize_varint(&mut connect_packet, hostname.len())?; // host length - 12
        connect_packet.write(hostname.as_bytes()).await?; // host name - shrecked.dev
        connect_packet.write_u16(port).await?; // port number - 42069
        connect_packet.write_u8(0x01).await?; // next state - 1 (ping)

        //println!("Sending: {:?}", connect_packet);

        send_prefixed_packet(&mut connection, &connect_packet).await?;

        let mut ping_packet: Vec<u8> = vec![];
        ping_packet.write_u8(0x00).await?;

        //println!("Sending: {:?}", ping_packet);

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

    async fn generate_contents(
        &mut self,
        connection: &mut TcpStream,
    ) -> Result<(), Box<dyn Error>> {
        println!("wut");

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

        let mut data = vec![0; len as usize];
        connection.read_exact(&mut data).await?;

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

async fn send_prefixed_packet(
    connection: &mut TcpStream,
    data: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    varint_rs::VarintWriter::write_usize_varint(&mut buffer, data.len())?;
    buffer.write(&data).await?;

    connection.write_all(&buffer).await?;

    Ok(())
}

async fn read_varint(stream: &mut TcpStream) -> Result<i32, Box<dyn Error>> {
    let mut buf = [0u8];
    let mut res = 0;
    let mut count = 0u32;
    loop {
        stream.read_exact(&mut buf).await?;
        res |= (buf[0] as i32 & (0b0111_1111 as i32))
            .checked_shl(7 * count)
            .ok_or("Unsupported protocol")?;

        count += 1;
        if count > 5 {
            break Err("Unsupported protocol".into());
        } else if (buf[0] & (0b1000_0000 as u8)) == 0 {
            break Ok(res);
        }
    }
}
