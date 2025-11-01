use beacon_config::Config;
use bevy_ecs::prelude::*;

use crate::net::{Game, Listener, Query, Rcon};

#[macro_use]
extern crate tracing;

mod net;

fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (mut world, mut schedule) = (World::new(), Schedule::default());
    let config = Config::setup(&mut world, &mut schedule, "beacon.toml")?;
    Game::setup(&mut world, &config.server)?;
    Rcon::setup(&mut world, &config.rcon)?;
    Query::setup(&mut world, &config.query)?;

    loop {
        schedule.run(&mut world);
    }
}
