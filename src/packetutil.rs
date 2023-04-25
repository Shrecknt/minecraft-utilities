use std::error::Error;

use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt}};

pub async fn send_prefixed_packet(
    connection: &mut TcpStream,
    data: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    varint_rs::VarintWriter::write_usize_varint(&mut buffer, data.len())?;
    buffer.write(&data).await?;

    connection.write_all(&buffer).await?;

    Ok(())
}

pub async fn read_varint(stream: &mut TcpStream) -> Result<i32, Box<dyn Error>> {
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
