use std::env::consts;

use beacon_config::{Config, ConfigActor};
use beacon_data::BEACON_VERSION;
use beacon_query::QueryActor;
use kameo::prelude::*;
use kameo_actors::{
    DeliveryStrategy,
    pubsub::{PubSub, Subscribe},
    scheduler::Scheduler,
};

use crate::error::BeaconError;

#[macro_use]
extern crate tracing;

mod error;

#[tokio::main]
async fn main() -> Result<(), BeaconError> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("starting beacon v{BEACON_VERSION}");
    warn!("beacon is in early development. expect bugs and incomplete features.");
    debug!(
        family = consts::FAMILY,
        os = consts::OS,
        arch = consts::ARCH,
        protocol = beacon_data::PROTOCOL_VERSION,
        supports = ?beacon_data::SUPPORTED_VERSIONS,
        debug = cfg!(debug_assertions),
        "build info"
    );

    // shared actors
    let scheduler = Scheduler::spawn_default();
    let config_update = PubSub::spawn(PubSub::new(DeliveryStrategy::BestEffort));

    // config watcher
    let config_path = "beacon.toml".into();
    let config = Config::read(&config_path)?;
    ConfigActor::spawn((config.clone(), config_path, config_update.clone()));

    // query protocol
    let query = QueryActor::spawn((config, scheduler.clone()));
    config_update.tell(Subscribe(query)).await?;

    // wait for ctrl-c
    tokio::signal::ctrl_c().await?;
    Ok(())
}
