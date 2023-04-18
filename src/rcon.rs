use std::error::Error;
use tokio::io::AsyncWriteExt;

pub struct RCONPacket {
    pub request_id: i32,
    pub request_type: i32,
    pub payload: Vec<u8>
}

impl RCONPacket {
    pub fn new() -> Self {
        RCONPacket {
            request_id: 0,
            request_type: 0,
            payload: vec![]
        }
    }

    pub async fn build(&self) -> Result<Vec<u8>, Box<dyn Error>> {
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

    pub async fn parse(&mut self, raw: &Vec<u8>) -> Result<(), Box<dyn Error>> {
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

    pub fn payload_to_string(&self) -> String {
        self.payload.iter().map(|x| char::from(*x)).collect()
    }

    pub fn print(&self) {
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
