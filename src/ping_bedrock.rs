use std::{error::Error, net::SocketAddr};

use rust_raknet::RaknetSocket;

#[derive(Debug, PartialEq)]
pub enum BedrockServerEdition {
    BedrockEdition,
    EducationEdition,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum BedrockServerGamemode {
    Creative,
    Survival,
    Adventure,
    Spectator,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PingBedrock {
    pub response_time: i64,
    pub edition: BedrockServerEdition,
    pub motd: String,
    pub protocol_version: i64,
    pub version_name: String,
    pub player_count: i64,
    pub max_player_count: i64,
    pub server_unique_id: String,
    pub game_mode: BedrockServerGamemode,
    pub game_mode_numeric: i8,
    pub port_v4: u16,
    pub port_v6: u16,
}

impl PingBedrock {
    pub async fn ping(addr: &SocketAddr) -> Result<Self, Box<dyn Error>> {
        let (latency, buf) = RaknetSocket::ping(addr).await.unwrap();
        let mut sections = buf.split(';');

        let edition = match sections.next().unwrap() {
            "MCPE" => BedrockServerEdition::BedrockEdition,
            "MCEE" => BedrockServerEdition::EducationEdition,
            _ => BedrockServerEdition::Unknown,
        };
        let mut motd = sections.next().unwrap().to_string();
        let protocol_version = i64::from_str_radix(sections.next().unwrap(), 10)?;
        let version_name = sections.next().unwrap().to_string();
        let player_count = i64::from_str_radix(sections.next().unwrap(), 10)?;
        let max_player_count = i64::from_str_radix(sections.next().unwrap(), 10)?;
        let server_unique_id = sections.next().unwrap().to_string();
        motd = format!("{motd}\n{}", sections.next().unwrap());
        let game_mode = match sections.next().unwrap() {
            "Creative" => BedrockServerGamemode::Creative,
            "Survival" => BedrockServerGamemode::Survival,
            "Adventure" => BedrockServerGamemode::Adventure,
            "Spectator" => BedrockServerGamemode::Spectator,
            _ => BedrockServerGamemode::Unknown,
        };
        let game_mode_numeric = i8::from_str_radix(sections.next().unwrap(), 10)?;
        let port_v4 = u16::from_str_radix(sections.next().unwrap(), 10)?;
        let port_v6 = u16::from_str_radix(sections.next().unwrap(), 10)?;

        Ok(PingBedrock {
            response_time: latency,
            edition,
            motd,
            protocol_version,
            version_name,
            player_count,
            max_player_count,
            server_unique_id,
            game_mode,
            game_mode_numeric,
            port_v4,
            port_v6,
        })
    }
}
