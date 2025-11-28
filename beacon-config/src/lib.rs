use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::def::*;

#[macro_use]
extern crate tracing;

mod def;

/// Message containing a config update
pub struct ConfigUpdate(pub Config);

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub world: WorldConfig,
    pub query: QueryConfig,
}

/// Load a section of the config, falling back to defaults on error
macro_rules! load {
    ($value:expr, $field:ident ($ty:ty)) => {
        let $field = $value
            .get(stringify!($field))
            .and_then(|v| <$ty>::deserialize(v.clone()).ok())
            .unwrap_or_else(|| {
                warn!(
                    "failed to parse {} config, using default",
                    stringify!($field)
                );
                <$ty>::default()
            });
    };
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("read error: {0}")]
    Read(#[from] toml::de::Error),
    #[error("write error: {0}")]
    Write(#[from] toml::ser::Error),
    #[error("update error: {0}")]
    Update(#[from] kameo::error::SendError<ConfigUpdate>),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl Config {
    /// Create a (default) config file at the given path
    fn create(path: &PathBuf) -> Result<Self, ConfigError> {
        let config = Self::default();
        let contents = toml::to_string_pretty(&config)?;
        fs::write(path, contents)?;
        Ok(config)
    }

    /// Read the config from the given path, creating it with defaults if it doesn't exist
    pub fn read(path: &PathBuf) -> Result<Self, ConfigError> {
        let value: toml::Value = match fs::read_to_string(path) {
            Ok(contents) if contents.is_empty() => return Self::create(path),
            Ok(contents) => toml::from_str(&contents)?,
            Err(_) => return Self::create(path),
        };

        // deserialize each section individually, falling back to defaults
        load!(value, server(ServerConfig));
        load!(value, world(WorldConfig));
        load!(value, query(QueryConfig));

        Ok(Self {
            server,
            world,
            query,
        })
    }
}
