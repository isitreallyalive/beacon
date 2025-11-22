use std::{env, io};

use crate::server::Beacon;

#[macro_use]
extern crate tracing;

/// The current version of beacon.
const BEACON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The protocol version corresponding with the Minecraft version that beacon targets.
/// This is currently Minecraft **1.21.9** and **1.21.10**.
///
/// See: https://minecraft.wiki/w/Protocol_version
const PROTOCOL_VERSION: u16 = 773;

mod server;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    info!("starting beacon v{BEACON_VERSION}");
    debug!(
        family = env::consts::FAMILY,
        os = env::consts::OS,
        arch = env::consts::ARCH,
        protocol = PROTOCOL_VERSION,
        debug = cfg!(debug_assertions),
        "build info"
    );

    let server = Beacon::new().await?;
    server.start().await;
    
    Ok(())
}
