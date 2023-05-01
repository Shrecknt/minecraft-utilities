// Crimsongale is mid

use std::error::Error;
use std::io::BufRead;
use std::time::Duration;

mod rcon;
use rcon::RconClient;

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

use tokio::time::timeout;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let check_version_number = "23w16a";
    println!(
        "{} = {}",
        check_version_number,
        parse_version(check_version_number).unwrap()
    );

    let args: Vec<String> = std::env::args().collect();

    let mut lookup_ip = "localhost";
    if args.len() == 2 {
        lookup_ip = args[1].as_str();
    }

    let mut address = ServerAddress::try_from(lookup_ip).unwrap();
    match resolve_address(&address).await {
        Ok(res) => {
            address = ServerAddress::from(res);
        }
        Err(err) => {
            println!("An error occured (1): {}", err);
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

    let mut protocol_ver: usize = 0;

    let addr = &format!("{}:{}", &address.host, &address.port);
    let test_ping = Ping::ping(addr, None, Some("shrecked.dev"), Some(42069));
    let ping_result = timeout(Duration::from_millis(1000), test_ping).await?;
    match ping_result {
        Ok(res) => {
            protocol_ver = Ping::get_protocol_version(&res)
                .unwrap_or(762)
                .try_into()
                .unwrap();
        }
        Err(err) => {
            println!("An error occured (4): {}", err);
        }
    }

    println!("Protocol version: {}", protocol_ver);

    let mut test_client = Client::connect(address.host, Some(address.port)).await?;
    let test_is_online_mode =
        test_client.check_online_mode(Some(protocol_ver), None, None, Some("gamer"));
    let is_online_mode_result = timeout(Duration::from_millis(1000), test_is_online_mode).await??;
    println!("Online mode results: {:?}", is_online_mode_result);

    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    let default_ip = "localhost:25575";
    println!("Enter the RCON server's ip (default '{}'):", default_ip);
    let input_ip = iterator.next().unwrap().unwrap();
    let mut ip = default_ip;
    if !input_ip.eq("") {
        ip = input_ip.as_str();
    }

    let mut connection = RconClient::connect(ip, None).await?;

    let default_password = "minecraft";
    println!(
        "RCON client connected! Enter password (default '{}'):",
        default_password
    );
    let input_password = iterator.next().unwrap().unwrap();
    let mut password = default_password;
    if !input_password.eq("") {
        password = input_password.as_str();
    }

    let return_packet = connection.login(password).await;

    match return_packet {
        Ok(_return_packet) => {
            println!("\x1b[32mConnected :)\x1b[0m");
            for line in iterator {
                let command = line.unwrap();

                let result = connection.command(command.as_str()).await;

                match result {
                    Ok(res) => {
                        println!("{}", res);
                    }
                    Err(err) => {
                        println!("\x1b[31mConnection failed:\x1b[0m {}", err);
                        return Ok(());
                    }
                }
            }
        }
        Err(err) => {
            println!("\x1b[31mConnection failed:\x1b[0m {}", err);
        }
    }

    Ok(())
}
