use std::env::consts;

use beacon::BeaconError;
use beacon_data::BEACON_VERSION;
use kameo::prelude::*;

use crate::server::BeaconActor;

#[macro_use]
extern crate tracing;

mod server;

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

    // run
    let server = BeaconActor::spawn("beacon.toml".into());
    server.wait_for_shutdown().await;
    Ok(())
}
