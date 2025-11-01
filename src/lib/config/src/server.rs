use std::net::{IpAddr, SocketAddr};

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Server {
    ip: IpAddr,
    #[serde(deserialize_with = "deserialize_port")]
    port: u16,
    motd: String,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".parse().unwrap(),
            port: 25565,
            motd: "A Beacon Server".to_string(),
        }
    }
}

fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;
    if port == 0 {
        return Err(serde::de::Error::custom("port must be between 1 and 65535"));
    }
    if (49152..=65535).contains(&port) {
        return Err(serde::de::Error::custom(
            "ephemeral ports (49152–65535) are not allowed",
        ));
    }
    if (1..=1023).contains(&port) {
        return Err(serde::de::Error::custom(
            "well-known ports (1–1023) are restricted",
        ));
    }
    Ok(port)
}

impl crate::Config {
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.server.ip, self.server.port)
    }
}
