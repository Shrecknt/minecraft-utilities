//! A low-level crate to send and receive Minecraft packets.
//!
//! You should probably use [`azalea`] or [`azalea_client`] instead, as
//! `azalea_protocol` delegates much of the work, such as auth, to the user of
//! the crate.
//!
//! [`azalea`]: https://crates.io/crates/azalea
//! [`azalea_client`]: https://crates.io/crates/azalea-client
//!
//! See [`crate::connect::Connection`] for an example.
//!
//! Thanks mat, 'your' code is now 'our' code
//!
//! yoinked from https://github.com/mat-1/azalea/blob/c1588ef66e844c067112ea880a54b4de9ec5a062/azalea-protocol/src/lib.rs

use std::{
    fmt::Display,
    net::{AddrParseError, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
};

/// A host and port. It's possible that the port doesn't resolve to anything.
///
/// # Examples
///
/// `ServerAddress` implements TryFrom<&str>, so you can use it like this:
/// ```
/// use minecraft_utilities::ServerAddress;
///
/// let addr = ServerAddress::try_from("localhost:25565").unwrap();
/// assert_eq!(addr.host, "localhost");
/// assert_eq!(addr.port, 25565);
/// ```
#[derive(Debug, Clone)]
pub struct ServerAddress {
    pub host: String,
    pub port: u16,
}

impl ServerAddress {
    pub fn new(host: &str, port: u16) -> Self {
        ServerAddress {
            host: host.to_string(),
            port,
        }
    }
}

impl<'a> TryFrom<&'a str> for ServerAddress {
    type Error = String;

    /// Convert a Minecraft server address (host:port, the port is optional) to
    /// a `ServerAddress`
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        if string.is_empty() {
            return Err("Empty string".to_string());
        }
        let mut parts = string.split(':');
        let host = parts.next().ok_or("No host specified")?.to_string();
        // default the port to 25565
        let port = parts.next().unwrap_or("25565");
        let port = u16::from_str(port).map_err(|_| "Invalid port specified")?;
        Ok(ServerAddress { host, port })
    }
}

impl From<SocketAddr> for ServerAddress {
    /// Convert an existing `SocketAddr` into a `ServerAddress`. This just
    /// converts the ip to a string and passes along the port. The resolver
    /// will realize it's already an IP address and not do any DNS requests.
    fn from(addr: SocketAddr) -> Self {
        ServerAddress {
            host: addr.ip().to_string(),
            port: addr.port(),
        }
    }
}

impl TryFrom<ServerAddress> for SocketAddr {
    type Error = AddrParseError;

    /// Convert an existing `ServerAddress` into a `SocketAddr`.
    fn try_from(addr: ServerAddress) -> Result<Self, Self::Error> {
        let v4_addr = Ipv4Addr::from_str(&addr.host)?;
        Ok(SocketAddr::V4(SocketAddrV4::new(v4_addr, addr.port)))
    }
}

impl Display for ServerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}
