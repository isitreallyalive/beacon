//! # beacon
//!
//! The Minecraft server that you'll love.

use beacon_core::BeaconServer;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let server = BeaconServer::new().await?;
    server.start().await
}
