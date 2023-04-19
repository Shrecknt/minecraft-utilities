use std::error::Error;
use std::io::BufRead;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

mod rcon;
use rcon::{RconPacket, RconClient};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = RconClient::connect("157.211.143.221:25575", None).await?;
    connection.login("minecraft").await?;
    let player_list_message = connection.command("list").await?;
    println!("Got player list:\n{}", player_list_message);

    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    let default_ip = "157.211.143.221:25575";
    println!("Enter the RCON server's ip (default '{}'):", default_ip);
    let input_ip = iterator.next().unwrap().unwrap();
    let mut ip = default_ip;
    if !input_ip.eq("") {
        ip = input_ip.as_str();
    }

    let mut stream = TcpStream::connect(ip).await?;

    let mut request_id: i32 = 0;

    let mut login_packet = RconPacket::new();
    login_packet.request_id = request_id;
    login_packet.request_type = 3;

    let default_password = "minecraft";
    println!("RCON client connected! Enter password (default '{}'):", default_password);
    login_packet.payload = default_password.into();
    let input_password = iterator.next().unwrap().unwrap();
    if !input_password.eq("") {
        login_packet.payload = input_password.into();
    }

    stream.write_all(&login_packet.build().await?).await?;

    let mut buf: Vec<u8> = vec![];
    stream.readable().await?;
    let _size = stream.try_read_buf(&mut buf).unwrap();
    let mut return_packet = RconPacket::new();
    return_packet.parse(&buf).await?;
    return_packet.print();
    if return_packet.request_id == -1 {
        println!("Connection failed :(");
    } else {
        println!("Connected :)");
    }

    for line in iterator {
        let command: Vec<u8> = line.unwrap().into();

        let mut send_packet = RconPacket::new();
        request_id += 1;
        send_packet.request_id = request_id;
        send_packet.request_type = 2;
        send_packet.payload = command;

        stream.write_all(&send_packet.build().await?).await?;

        let mut res_buf: Vec<u8> = vec![];
        stream.readable().await?;
        let _size = stream.read_buf(&mut res_buf).await?;
        let mut res_return_packet = RconPacket::new();
        res_return_packet.parse(&res_buf).await?;
        res_return_packet.print();
    }

    Ok(())
}
