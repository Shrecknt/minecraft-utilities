use std::error::Error;

use tokio::net::TcpStream;

pub struct Client {
    host: String,
    port: u16,
    connection: Option<TcpStream>,
}

impl Client {
    pub fn new(host: String, port: Option<u16>) -> Self {
        Client {
            host,
            port: port.unwrap_or(25565),
            connection: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        if self.connection.is_some() {
            return Err("Already connected".into());
        }
        self.connection = Some(TcpStream::connect(format!("{}:{}", self.host, self.port)).await?);
        Ok(())
    }
}
