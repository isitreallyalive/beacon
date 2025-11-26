use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{ConfigError, def::*};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    pub fn read(path: &PathBuf) -> Result<Self, ConfigError> {
        // read the file contents
        let value: toml::Value = match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents)?,
            Err(_) => {
                // if the file doesn't exist, create one with defaults
                let default = Self::default();
                let contents = toml::to_string_pretty(&default)?;
                fs::write(path, contents)?;
                return Ok(default);
            }
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

    /// Reload the config from the given path
    pub(crate) fn reload(&mut self, path: &PathBuf) -> Result<(), ConfigError> {
        *self = Self::read(path)?;
        Ok(())
    }
}
