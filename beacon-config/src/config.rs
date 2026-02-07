use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use bevy_ecs::prelude::*;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

use crate::ConfigError;

const DEFAULT_CONFIG: &str = include_str!("../../assets/beacon.default.toml");

/// The configuration for the server.
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    /// The host to bind to.
    pub ip: Ipv4Addr,
    /// The port to bind to.
    pub port: u16,
    /// Whether the server should report its status.
    // todo: use
    pub status: bool,
    /// The path to the server icon.
    pub icon: PathBuf,
    /// The Message of the Day
    pub motd: String,
    /// The maximum number of players allowed on the server.
    pub max_players: u32,
}

// todo: proper error handling for incorrect fields
impl Config {
    /// Load the configuration from a file, with defaults.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config: Config = Figment::new()
            // load the default configuration
            .merge(Toml::string(DEFAULT_CONFIG))
            // override it with the user's configuration
            .merge(Toml::file(path))
            .extract()?;

        crate::favicon::load(&config.server.icon)?;
        Ok(config)
    }
}
