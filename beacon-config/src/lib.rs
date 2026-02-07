//! # beacon-config
//!
//! Contains all configuration structs, and hot-reload logic.

use std::path::Path;

use bevy_ecs::prelude::*;

use miette::Diagnostic;
use thiserror::Error;

pub use crate::config::Config;
pub use crate::favicon::*;
use crate::reload::ConfigManager;

mod config;
mod favicon;
mod reload;

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

    /// An error occurred while managing the favicon.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Favicon(#[from] favicon::FaviconError),
}

/// Add configuration to the ECS.
pub fn ecs<P: AsRef<Path>>(
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
