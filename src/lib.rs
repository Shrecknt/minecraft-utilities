#![doc = include_str!("../README.md")]

mod packetutil;

mod rcon;
pub use rcon::{RconClient, RconError};

mod ping;
pub use ping::Ping;

mod ping_bedrock;
pub use ping_bedrock::{BedrockServerEdition, BedrockServerGamemode, PingBedrock};

mod client;
pub use client::{Client, OnlineModeResults};

mod server_address;
pub use server_address::ServerAddress;

mod resolve_address;
pub use resolve_address::resolve_address;

mod versions;
pub use versions::parse_version;
