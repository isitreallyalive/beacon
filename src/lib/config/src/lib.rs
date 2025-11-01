use std::path::Path;

use bevy_ecs::prelude::*;

#[macro_use]
extern crate tracing;

mod server;
mod watcher;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to watch config file: {0}")]
    WatchError(#[from] notify::Error),
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Resource, serde::Deserialize, Default, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub server: server::Server,
}

impl Config {
    fn read(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        Ok(config)
    }

    pub fn setup<P>(world: &mut World, schedule: &mut Schedule, path: P) -> Result<(), ConfigError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let config = Config::read(path)?;
        world.insert_resource(config);
        watcher::ConfigWatcher::setup(world, schedule, path)?;
        Ok(())
    }
}
