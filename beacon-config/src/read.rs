use std::{default, fs, path::PathBuf};

use serde::Deserialize;

use crate::{ConfigError, def::*};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigData {
    pub server: ServerConfig,
    pub world: WorldConfig,
    pub query: QueryConfig,
}

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

macro_rules! changed {
    ($old:expr => $new:expr, $field:ident) => {
        if $old.$field != $new.$field {
            info!("reloaded {} config", stringify!($field));
            $old.$field = $new.$field;
        }
    };
}

impl ConfigData {
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

    pub fn reload(&mut self, path: &PathBuf) -> Result<(), ConfigError> {
        let new = Self::read(path)?;
        changed!(self => new, server);
        changed!(self => new, world);
        changed!(self => new, query);
        Ok(())
    }
}
