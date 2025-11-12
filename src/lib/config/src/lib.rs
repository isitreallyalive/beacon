use std::path::Path;

use bevy_ecs::prelude::*;
use serde::Deserialize;

#[macro_use]
extern crate tracing;

mod watcher;

/// Version of Minecraft that this server is compatible with
pub const VERSION: &str = "1.21.10";
/// Protocol version of Minecraft that this server is compatible with
pub const PROTOCOL_VERSION: u32 = 773;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to watch config file: {0}")]
    WatchError(#[from] notify::Error),
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Resource, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub world: WorldConfig,
    pub query: QueryConfig,
    pub rcon: RconConfig,
    pub msmp: MsmpConfig,
}

impl Config {
    fn read(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        Ok(config)
    }

    pub fn setup<P>(
        world: &mut World,
        schedule: &mut Schedule,
        path: P,
    ) -> Result<Config, ConfigError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let config = Config::read(path)?;
        world.insert_resource(config.clone());
        watcher::ConfigWatcher::setup(world, schedule, path)?;
        Ok(config)
    }
}

macro_rules! config {
    (
        $(
            $(#[$struct_meta:meta])*
            struct $name:ident {
                $(
                    $(#[$field_meta:meta])*
                    $field:ident : $type:ty = $default:expr
                ),*$(,)?
            }
        )+
    ) => {
        $(
            $(#[$struct_meta])*
            #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
            #[serde(default)]
            pub struct $name {
                $(
                    $(#[$field_meta])*
                    pub $field: $type,
                )*
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )*
                    }
                }
            }
        )+
    };
}

// todo: the rest of server.properties
// https://minecraft.wiki/w/Server.properties
//
// accepts-transfers=false
// allow-flight=false
// broadcast-console-to-ops=true
// broadcast-rcon-to-ops=true
// bug-report-link=
// difficulty=easy
// enable-code-of-conduct=false
// enable-jmx-monitoring=false
// enable-status=true
// enforce-secure-profile=true
// enforce-whitelist=false
// entity-broadcast-range-percentage=100
// force-gamemode=false
// function-permission-level=2
// gamemode=survival
// generate-structures=true
// generator-settings={}
// hardcore=false
// hide-online-players=false
// initial-disabled-packs=
// initial-enabled-packs=vanilla
// level-seed=
// level-type=minecraft\:normal
// log-ips=true
// management-server-secret=[Random text]
// management-server-tls-enabled=true
// management-server-tls-keystore=
// management-server-tls-keystore-password=
// max-chained-neighbor-updates=1000000
// max-tick-time=60000
// max-world-size=29999984
// network-compression-threshold=256
// online-mode=true
// op-permission-level=4
// pause-when-empty-seconds=60
// player-idle-timeout=0
// prevent-proxy-connections=false
// rate-limit=0
// rcon.password=
// region-file-compression=deflate
// require-resource-pack=false
// resource-pack=
// resource-pack-id=
// resource-pack-prompt=
// resource-pack-sha1=
// simulation-distance=10
// spawn-protection=16
// status-heartbeat-interval=0
// sync-chunk-writes=true
// text-filtering-config=
// text-filtering-version=0
// use-native-transport=true
// view-distance=10
// white-list=false

config! {
    struct ServerConfig {
        /// The IP address the server listens on. If empty, the server listens on all available IP addresses.
        /// It is recommended to leave this empty.
        ip: std::net::IpAddr = "0.0.0.0".parse().unwrap(),

        /// The TCP port the server listens on.
        #[serde(deserialize_with = "deserialize_port")]
        port: u16 = 25565,

        /// Message of the day, displayed to clients when they connect.
        motd: String = "A Beacon Server".to_string(),

        /// Maximum number of players allowed on the server.
        max_players: u32 = 20
    }

    struct WorldConfig {
        /// Name of the world folder
        name: String = "world".to_string(),
    }

    struct QueryConfig {
        /// Whether to enable the query protocol, which allows clients to get server information over UDP.
        enable: bool = false,

        /// The IP address the query protocol listens on. If empty, it listens on all available IP addresses.
        ip: std::net::IpAddr = "0.0.0.0".parse().unwrap(),

        /// The UDP port the query protocol listens on.
        #[serde(deserialize_with = "deserialize_port")]
        port: u16 = 25565,
    }

    struct RconConfig {
        /// Whether to enable rcon, which allows access to the server console over a network.
        enable: bool = false,

        /// The IP address rcon listens on. If empty, RCON only listens to localhost.
        ip: std::net::IpAddr = "127.0.0.1".parse().unwrap(),

        /// The TCP port rcon listens on.
        #[serde(deserialize_with = "deserialize_port")]
        port: u16 = 25575,
    }

    struct MsmpConfig {
        /// Whether to enable MSMP, which allows server management over a network.
        enable: bool = false,

        /// The IP address MSMP listens on. If empty, MSMP only listens to localhost.
        ip: std::net::IpAddr = "127.0.0.1".parse().unwrap(),

        /// The TCP port MSMP listens on.
        #[serde(deserialize_with = "deserialize_port")]
        port: u16 = 8555,
    }
}

// todo: duplicate ports are not allowed
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
