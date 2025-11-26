use beacon_config::ConfigError;
use kameo::prelude::*;
use kameo_actors::pubsub::Subscribe;

#[derive(Debug, thiserror::Error)]
pub enum BeaconError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
}

impl<A> From<SendError<Subscribe<A>>> for BeaconError
where
    A: Actor,
{
    fn from(err: SendError<Subscribe<A>>) -> Self {
        BeaconError::Config(ConfigError::Subscribe(err.to_string()))
    }
}
