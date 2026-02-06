//! # beacon-config
//!
//! Contains all configuration structs, and hot-reload logic.

use std::{net::Ipv4Addr, path::Path};

use bevy_ecs::prelude::*;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

use crate::reload::ConfigManager;

mod reload;

const DEFAULT_CONFIG: &str = include_str!("../../assets/default-config.toml");

/// Errors that can occur while managing configuration.
#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    /// An error occurred while watching the configuration file for changes.
    #[error("failed to watch configuration file for changes")]
    #[diagnostic(help("check that the file exists and is accessible"))]
    Notify(#[from] notify::Error),

    /// An error occurred while reading the configuration file.
    #[error("failed to read configuration file")]
    #[diagnostic(help("check that the file exists and is a valid TOML file"))]
    Read(#[from] figment::Error),
}

/// The configuration for the server.
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct Config {
    /// The host to bind to.
    pub host: Ipv4Addr,
    /// The port to bind to.
    pub port: u16,
}

// todo: proper error handling for incorrect fields
impl Config {
    /// Load the configuration from a file, with defaults.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config = Figment::new()
            // load the default configuration
            .merge(Toml::string(DEFAULT_CONFIG))
            // override it with the user's configuration
            .merge(Toml::file(path))
            .extract()?;
        Ok(config)
    }
}

/// Add configuration to the ECS.
pub fn register<P: AsRef<Path>>(
    world: &mut World,
    schedule: &mut Schedule,
    path: P,
) -> Result<Config, ConfigError> {
    let config = Config::load(&path)?;
    world.insert_resource(config.clone());

    world.insert_resource(ConfigManager::new(path)?);
    schedule.add_systems(ConfigManager::reload);

    Ok(config)
}
