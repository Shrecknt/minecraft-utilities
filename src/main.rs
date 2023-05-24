// Crimsongale is mid

use std::str::FromStr;
use std::time::Duration;
use std::{error::Error, net::SocketAddr};

mod ping;
use ping::Ping;

mod client;
use client::Client;

mod server_address;
use server_address::ServerAddress;

mod resolve_address;
use resolve_address::resolve_address;

mod packetutil;

mod versions;
use versions::parse_version;

mod ping_bedrock;
use ping_bedrock::{BedrockServerEdition, PingBedrock};

use tokio::time::timeout;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let bedrock_addr = SocketAddr::from_str("127.0.0.1:19132")?;
    let bedrock_ping = PingBedrock::ping(&bedrock_addr);
    let bedrock_ping_timeout = timeout(Duration::from_millis(2500), bedrock_ping).await;
    match bedrock_ping_timeout {
        Ok(bedrock_ping_res) => match bedrock_ping_res {
            Ok(bedrock_ping_res) => {
                println!("test: {bedrock_ping_res:?}");
                if bedrock_ping_res.edition == BedrockServerEdition::BedrockEdition {
                    println!("Bedrock edition!");
                }
            }
            Err(err) => {
                println!("An error occured (1): {}", err);
            }
        },
        Err(err) => {
            println!("An error occured (1): {}", err);
        }
    }

    let check_version_number = "23w16a";
    println!(
        "{} = {}",
        check_version_number,
        parse_version(check_version_number)?
    );

    let args: Vec<String> = std::env::args().collect();

    let mut lookup_ip = "localhost";
    if args.len() == 2 {
        lookup_ip = args[1].as_str();
    }

    let mut address = ServerAddress::try_from(lookup_ip)?;
    match resolve_address(&address).await {
        Ok(res) => {
            address = ServerAddress::from(res);
        }
        Err(err) => {
            println!("An error occured (2): {}", err);
        }
    }
    println!("Resolved address: {:?}", address);

    // let mut test_client = Client::connect(address.host.clone(), Some(address.port.clone())).await?;
    // let mut future = Client::connect(address.host.clone(), Some(address.port)).await?;
    // let _test_client_async = tokio::spawn(async move {
    //     let res = future
    //         .check_online_mode(None, None, None, Some("gamer"))
    //         .await;
    //     match res {
    //         Ok(result) => {
    //             println!("Online mode: {:?}", result);

    //             // let print: String = data.iter().map(|x| char::from(*x)).collect();
    //             // println!("{}", print);
    //         }
    //         Err(err) => {
    //             println!("An error occured (2): {}", err);
    //         }
    //     }
    // });
    // let connected = test_client.join().await;
    // match connected {
    //     Ok(data) => {
    //         println!("Connected! {} {:?}", data.response_id, data.buffer);

    //         let print: String = data.buffer.iter().map(|x| char::from(*x)).collect();
    //         println!("{}", print);
    //     }
    //     Err(err) => {
    //         println!("An error occured (3): {}", err);
    //     }
    // }

    let mut protocol_ver: i32 = 0;

    let test_ping = Ping::ping(
        &address.host,
        Some(address.port),
        None,
        Some("shrecked.dev"),
        Some(42069),
    );
    let ping_result = timeout(Duration::from_millis(1000), test_ping).await;
    match ping_result {
        Ok(ping_result) => match ping_result {
            Ok(res) => protocol_ver = Ping::get_protocol_version(&res)?,
            Err(err) => {
                println!("An error occured (4): {}", err);
            }
        },
        Err(err) => {
            println!("An error occured (4): {}", err);
        }
    }

    let test_ping_2 = Ping::ping_legacy_protocol(
        &address.host,
        Some(address.port),
        Some(0x49),
        Some("localhost"),
        Some(25565),
    );
    let ping_result_2 = timeout(Duration::from_millis(10000), test_ping_2).await;
    match ping_result_2 {
        Ok(ping_result_2) => match ping_result_2 {
            Ok(res) => {
                println!("res: {res:?}");
            }
            Err(err) => {
                println!("An error occured (5): {}", err);
            }
        },
        Err(err) => {
            println!("An error occured (5): {}", err);
        }
    }

    println!("Protocol version: {}", protocol_ver);

    let mut test_client = Client::connect(&address.host, Some(address.port)).await?;
    let test_is_online_mode =
        test_client.check_online_mode(Some(protocol_ver), None, None, Some("gamer"));
    let (is_online_mode_result, other) =
        timeout(Duration::from_millis(1000), test_is_online_mode).await??;
    println!(
        "Online mode results: {:?}, other: {}",
        is_online_mode_result,
        other.unwrap_or("No other information".to_string())
    );

    Ok(())
}
