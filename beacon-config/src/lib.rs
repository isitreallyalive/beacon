//! # beacon-config
//!
//! Contains all configuration structs, and hot-reload logic.

use std::{net::Ipv4Addr, path::Path};

use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("../../assets/default-config.toml");

/// The configuration for the server.
#[derive(Deserialize)]
pub struct Config {
    /// The host to bind to.
    pub host: Ipv4Addr,
    /// The port to bind to.
    pub port: u16,
}

// todo: proper error handling for incorrect fields
impl Config {
    /// Load the configuration from a file, with defaults.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, figment::Error> {
        Figment::new()
            // load the default configuration
            .merge(Toml::string(DEFAULT_CONFIG))
            // override it with the user's configuration
            .merge(Toml::file(path))
            .extract()
    }
}
