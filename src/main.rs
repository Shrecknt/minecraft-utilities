use std::error::Error;
use std::io::BufRead;

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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
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

    let mut test_client = Client::new(lookup_ip.to_string(), None);
    let mut future = Client::new(lookup_ip.to_string(), None);
    let _test_client_async = tokio::spawn(async move {
        let res = future.connect().await;
        match res {
            Ok(()) => {
                println!("Ok! :D");
            }
            Err(err) => {
                println!("An error occured (2): {}", err);
            }
        }
    });
    let connected = test_client.connect().await;
    match connected {
        Ok(()) => {
            println!("Connected!");
        }
        Err(err) => {
            println!("An error occured (3): {}", err);
        }
    }

    let test = Ping::ping(lookup_ip, None, None, None).await;
    match test {
        Ok(res) => {
            println!("Got result: {}", json::stringify_pretty(res.contents, 4));
        }
        Err(err) => {
            println!("An error occured (4): {}", err);
        }
    }

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
