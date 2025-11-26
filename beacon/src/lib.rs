use beacon_config::ConfigUpdate;
use kameo::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum BeaconError {
    #[error("file watcher error: {0}")]
    Watcher(#[from] notify::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to send config update: {0}")]
    ConfigUpdate(#[from] SendError<ConfigUpdate>),
}
