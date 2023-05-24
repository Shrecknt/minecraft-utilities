use std::error::Error;

use crate::ServerAddress;
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
    pub async fn ping(addr: &ServerAddress) -> Result<Self, Box<dyn Error>> {
        let deref_addr: std::net::SocketAddr = addr.clone().try_into()?;
        let (latency, buf);
        match RaknetSocket::ping(&deref_addr).await {
            Ok(v) => {
                (latency, buf) = v;
            }
            Err(_err) => {
                return Err("Error connecting to server".into());
            }
        };
        let mut sections = buf.split(';');

        // potential issue: what if motd has ';' character?
        if sections.clone().collect::<Vec<_>>().len() != 14 {
            return Err("Received invalid data".into());
        }

        let edition = match sections.next().unwrap_or("?") {
            "MCPE" => BedrockServerEdition::BedrockEdition,
            "MCEE" => BedrockServerEdition::EducationEdition,
            _ => BedrockServerEdition::Unknown,
        };
        let mut motd = sections.next().unwrap_or("?").to_string();
        let protocol_version = sections.next().unwrap_or("-1").parse::<i64>()?;
        let version_name = sections.next().unwrap_or("?").to_string();
        let player_count = sections.next().unwrap_or("-1").parse::<i64>()?;
        let max_player_count = sections.next().unwrap_or("-1").parse::<i64>()?;
        let server_unique_id = sections.next().unwrap_or("?").to_string();
        motd = format!("{motd}\n{}", sections.next().unwrap_or("?"));
        let game_mode = match sections.next().unwrap_or("?") {
            "Creative" => BedrockServerGamemode::Creative,
            "Survival" => BedrockServerGamemode::Survival,
            "Adventure" => BedrockServerGamemode::Adventure,
            "Spectator" => BedrockServerGamemode::Spectator,
            _ => BedrockServerGamemode::Unknown,
        };
        let game_mode_numeric = sections.next().unwrap_or("-1").parse::<i8>()?;
        let port_v4 = sections.next().unwrap_or("0").parse::<u16>()?;
        let port_v6 = sections.next().unwrap_or("0").parse::<u16>()?;

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
