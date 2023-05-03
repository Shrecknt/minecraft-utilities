use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub struct MinecraftPacket {
    pub buffer: Vec<u8>,
    pub packet_id: i32,
}

pub async fn send_prefixed_packet(
    connection: &mut TcpStream,
    data: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    write_varint(&mut buffer, i32::try_from(data.len())?).await?;
    buffer.write_all(data).await?;

    connection.write_all(&buffer).await?;

    Ok(())
}

pub async fn get_packet(connection: &mut TcpStream) -> Result<MinecraftPacket, Box<dyn Error>> {
    get_insane_packet(connection, 16777216).await
}

pub async fn get_insane_packet(
    connection: &mut TcpStream,
    sanity_limit: i32,
) -> Result<MinecraftPacket, Box<dyn Error>> {
    connection.readable().await?;
    let len = read_varint(connection).await?;
    if len > sanity_limit {
        return Err("Someone is trying to DDOS you or something :eyes: (packet size varint exceeded sanity check)".into());
    }
    let mut len_usize: usize = len.try_into()?;
    let (packet_id_len, packet_id_data) = read_varint_len(connection).await?;
    let packet_id_length: usize = packet_id_len.try_into()?;
    len_usize -= packet_id_length;
    let mut res: Vec<u8> = vec![0; len_usize];
    connection.read_exact(&mut res).await?;
    Ok(MinecraftPacket {
        packet_id: packet_id_data,
        buffer: res,
    })
}

pub async fn read_varint(stream: &mut TcpStream) -> Result<i32, Box<dyn Error>> {
    let (_len, data) = read_varint_len(stream).await?;
    Ok(data)
}

// yoinked from https://github.com/mat-1/azalea/blob/1fb4418f2c9cbd004c64c2f23d2d0352ee12c0e5/azalea-buf/src/write.rs#L36
// thanks mat <3
pub async fn write_varint(buf: &mut impl std::io::Write, val: i32) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0];
    let mut value = val;
    if value == 0 {
        buf.write_all(&buffer)?;
    }
    while value != 0 {
        buffer[0] = (value & 0b0111_1111) as u8;
        value = (value >> 7) & (i32::max_value() >> 6);
        if value != 0 {
            buffer[0] |= 0b1000_0000;
        }
        buf.write_all(&buffer)?;
    }
    Ok(())
}

pub async fn read_varint_len(stream: &mut TcpStream) -> Result<(u32, i32), Box<dyn Error>> {
    let mut buf = [0u8];
    let mut res = 0;
    let mut count = 0u32;

    loop {
        stream.readable().await?;
        stream.read_exact(&mut buf).await?;
        res |= (buf[0] as i32 & (0b0111_1111_i32))
            .checked_shl(7 * count)
            .ok_or("Unsupported protocol")?;

        count += 1;
        if count > 5 {
            break Err("Unsupported protocol".into());
        } else if (buf[0] & (0b1000_0000_u8)) == 0 {
            break Ok((count, res));
        }
    }
}

pub async fn read_varint_buf(stream: &[u8]) -> Result<(u32, i32), Box<dyn Error>> {
    let mut buf = vec![0u8];
    let mut res = 0;
    let mut count = 0u32;

    let mut idx = 0;

    loop {
        buf[0] = stream[idx];
        idx += 1;
        res |= (buf[0] as i32 & (0b0111_1111_i32))
            .checked_shl(7 * count)
            .ok_or("Unsupported protocol")?;

        count += 1;
        if count > 5 {
            break Err("Unsupported protocol".into());
        } else if (buf[0] & (0b1000_0000_u8)) == 0 {
            break Ok((count, res));
        }
    }
}
