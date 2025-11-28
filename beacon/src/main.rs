use beacon::BeaconError;
use kameo::prelude::*;

use crate::server::BeaconActor;

#[macro_use]
extern crate tracing;

mod server;

#[tokio::main]
async fn main() -> Result<(), BeaconError> {
    beacon_tui::register()?;
    BeaconActor::spawn("beacon.toml".into())
        .wait_for_shutdown()
        .await;
    Ok(())
}
