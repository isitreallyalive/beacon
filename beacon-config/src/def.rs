use std::net::Ipv4Addr;

const ALL_INTERFACES: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerConfig {
    #[serde(skip_serializing)]
    pub ip: Ipv4Addr,
    pub port: u16,
    pub motd: String,
    pub max_players: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: ALL_INTERFACES,
            port: 25565,
            motd: "A Beacon Server".to_string(),
            max_players: 20,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldConfig {
    pub name: String,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct QueryConfig {
    #[serde(skip_serializing)]
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            ip: ALL_INTERFACES,
            port: 25565,
        }
    }
}
