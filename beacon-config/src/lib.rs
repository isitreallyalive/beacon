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

impl Config {
    /// Read the config from the given path, creating it with defaults if it doesn't exist
    pub fn read(path: &PathBuf) -> Self {
        // read the file contents
        let value: toml::Value = match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(v) => v,
                Err(_) => {
                    warn!("failed to parse config file, using defaults");
                    return Self::default();
                }
            },
            Err(_) => {
                // if the file doesn't exist, create one with defaults
                let default = Self::default();
                match toml::to_string_pretty(&default) {
                    Ok(contents) => {
                        let _ = fs::write(path, contents);
                    }
                    Err(e) => {
                        warn!("failed to write default config file: {}", e);
                    }
                }
                return default;
            }
        };

        // deserialize each section individually, falling back to defaults
        load!(value, server(ServerConfig));
        load!(value, world(WorldConfig));
        load!(value, query(QueryConfig));

        Self {
            server,
            world,
            query,
        }
    }
}
