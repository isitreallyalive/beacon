//! # beacon
//!
//! The Minecraft server that you'll love.

use std::env::consts;

use beacon_core::BeaconServer;
use miette::Result;
use tracing_subscriber::{fmt::time::ChronoLocal, layer::SubscriberExt, util::SubscriberInitExt};

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<()> {
    // configure tracing
    tracing_subscriber::registry()
        // env filter
        .with(tracing_subscriber::EnvFilter::from("debug"))
        // pretty
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(ChronoLocal::new("%d/%m/%Y %H:%M:%S".to_string())),
        )
        .init();

    // startup logging
    info!("starting beacon v{}", beacon_data::BEACON_VERSION);
    warn!("beacon is in early development. expect bugs and incomplete features.");
    debug!(
        family = consts::FAMILY,
        os = consts::OS,
        arch = consts::ARCH,
        debug = cfg!(debug_assertions),
        protocol = beacon_data::PROTOCOL_VERSION,
        supports = tracing::field::display(format!(
            "[{}]",
            beacon_data::SUPPORTED_VERSIONS
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    );

    let mut server = BeaconServer::new("beacon.toml").await?;
    server.start().await
}
