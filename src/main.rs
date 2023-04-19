use std::error::Error;
use std::io::BufRead;

mod rcon;
use rcon::RconClient;

mod ping;
use ping::Ping;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {

    let test = Ping::ping("157.211.143.221:25565").await;
    match test {
        Ok(res) => {
            println!("Got result: {}", json::stringify_pretty(res.contents, 4));
        }
        Err(err) => {
            println!("An error occured: {}", err);
        }
    }

    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    let default_ip = "157.211.143.221:25575";
    println!("Enter the RCON server's ip (default '{}'):", default_ip);
    let input_ip = iterator.next().unwrap().unwrap();
    let mut ip = default_ip;
    if !input_ip.eq("") {
        ip = input_ip.as_str();
    }

    let mut connection = RconClient::connect(ip, None).await?;

    let default_password = "minecraft";
    println!("RCON client connected! Enter password (default '{}'):", default_password);
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
