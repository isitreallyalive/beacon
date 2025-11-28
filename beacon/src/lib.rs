use std::sync::PoisonError;

#[derive(Debug, thiserror::Error)]
pub enum BeaconError {
    #[error("file watcher error: {0}")]
    Watcher(#[from] notify::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {0}")]
    Config(#[from] beacon_config::ConfigError),

    #[error("poison error: {0}")]
    Poison(String),

    #[error("{0}")]
    Other(&'static str),
}

impl<T> From<PoisonError<T>> for BeaconError {
    fn from(err: PoisonError<T>) -> Self {
        BeaconError::Poison(err.to_string())
    }
}
