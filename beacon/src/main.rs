use std::{env, io};

use crate::server::Beacon;
use beacon_data::BEACON_VERSION;

#[macro_use]
extern crate tracing;

mod server;

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

    let server = Beacon::new().await?;
    server.start().await;

    Ok(())
}
