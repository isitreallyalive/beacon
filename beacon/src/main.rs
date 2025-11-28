use beacon::BeaconError;
use kameo::prelude::*;

use crate::server::BeaconActor;

#[macro_use]
extern crate tracing;

mod server;

#[tokio::main]
async fn main() -> Result<(), BeaconError> {
    // run
    let server = BeaconActor::spawn("beacon.toml".into());
    server.wait_for_shutdown().await;

    Ok(())
}
