use std::error::Error;
use std::io::BufRead;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::{net::TcpStream};

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

pub struct RCONPacket {
    request_id: i32,
    request_type: i32,
    payload: Vec<u8>
}

impl RCONPacket {
    pub fn new() -> Self {
        RCONPacket {
            request_id: 0,
            request_type: 0,
            payload: vec![]
        }
    }

    async fn build(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut packet: Vec<u8> = vec![];
        packet.write_i32_le(self.request_id as i32).await?;
        packet.write_i32_le(self.request_type as i32).await?; // 3 for login, 2 to run a command, 0 for a multi-packet response
        packet.write(&self.payload).await?;
        packet.write(b"\0\0").await?;
        let mut buffer: Vec<u8> = vec![];
        buffer.write_i32_le(packet.len() as i32).await?;
        buffer.write(&packet).await?;

        Ok(buffer)
    }

    async fn parse(&mut self, raw: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let length = as_i32_le(&raw[0..4].try_into().unwrap());
        println!("Packet Length: {}", length);
        let request_id = as_i32_le(&raw[4..8].try_into().unwrap());
        let request_type = as_i32_le(&raw[8..12].try_into().unwrap());
        let mut payload = (&raw[13..]).to_vec();
        payload.pop();
        payload.pop();

        self.request_id = request_id;
        self.request_type = request_type;
        self.payload = payload;

        Ok(())
    }

    fn payload_to_string(&self) -> String {
        self.payload.iter().map(|x| char::from(*x)).collect()
    }

    fn print(&self) {
        let str = self.payload_to_string();
        println!("Request ID: {}, Request Type: {}, payload: {}", self.request_id, self.request_type, str);
    }
}

fn as_i32_le(array: &[u8; 4]) -> i32 {
    ((array[0] as i32) <<  0) +
    ((array[1] as i32) <<  8) +
    ((array[2] as i32) << 16) +
    ((array[3] as i32) << 24)
}
