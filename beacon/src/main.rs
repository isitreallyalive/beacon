use std::{env, io};

use beacon_config::ConfigActor;
use beacon_data::BEACON_VERSION;
use kameo::prelude::*;
use kameo_actors::{DeliveryStrategy, pubsub::PubSub};

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("starting beacon v{BEACON_VERSION}");
    warn!("beacon is in early development. expect bugs and incomplete features.");
    debug!(
        family = env::consts::FAMILY,
        os = env::consts::OS,
        arch = env::consts::ARCH,
        protocol = beacon_data::PROTOCOL_VERSION,
        supports = ?beacon_data::SUPPORTED_VERSIONS,
        debug = cfg!(debug_assertions),
        "build info"
    );

    let config_pubsub = PubSub::spawn(PubSub::new(DeliveryStrategy::BestEffort));
    ConfigActor::spawn(("beacon.toml".into(), config_pubsub));

    // wait for ctrl-c
    tokio::signal::ctrl_c().await
}
