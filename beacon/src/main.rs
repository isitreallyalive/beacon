use std::{env, io};

use beacon_data::{BEACON_VERSION, PROTOCOL_VERSION, SUPPORTED_VERSION};

use crate::server::Beacon;

#[macro_use]
extern crate tracing;

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
        supports = SUPPORTED_VERSION,
        debug = cfg!(debug_assertions),
        "build info"
    );

    let server = Beacon::new().await?;
    server.start().await;

    Ok(())
}
