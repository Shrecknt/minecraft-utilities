use std::error::Error;
use std::io::BufRead;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

mod rcon;
use rcon::RCONPacket;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    let mut stream = TcpStream::connect("157.211.143.221:25575").await?;

    let mut request_id: i32 = 0;

    let mut login_packet = RCONPacket::new();
    login_packet.request_id = request_id;
    login_packet.request_type = 3;
    println!("RCON client connected! Enter password:");
    login_packet.payload = iterator.next().unwrap().unwrap().into();
    stream.write_all(&login_packet.build().await?).await?;

    let mut buf: Vec<u8> = vec![];
    stream.readable().await?;
    let _size = stream.try_read_buf(&mut buf).unwrap();
    let mut return_packet = RCONPacket::new();
    return_packet.parse(&buf).await?;
    return_packet.print();
    if return_packet.request_id == -1 {
        println!("Connection failed :(");
    } else {
        println!("Connected :)");
    }

    for line in iterator {
        let command: Vec<u8> = line.unwrap().into();

        let mut send_packet = RCONPacket::new();
        request_id += 1;
        send_packet.request_id = request_id;
        send_packet.request_type = 2;
        send_packet.payload = command;

        stream.write_all(&send_packet.build().await?).await?;

        let mut res_buf: Vec<u8> = vec![];
        stream.readable().await?;
        let _size = stream.read_buf(&mut res_buf).await?;
        let mut res_return_packet = RCONPacket::new();
        res_return_packet.parse(&res_buf).await?;
        res_return_packet.print();
    }

    Ok(())
}
