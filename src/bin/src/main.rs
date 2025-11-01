use beacon_config::Config;
use bevy_ecs::prelude::*;

use crate::net::{
    conn::{self, Connection},
    listen,
};

mod net;

fn main() -> Result<()> {
    // setup logging
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // start bevy
    let (mut world, mut schedule) = (World::new(), Schedule::default());
    let config = Config::setup(&mut world, &mut schedule, "beacon.toml")?;

    // setup listeners
    world.insert_resource(listen::GameListener::new(&config)?);
    schedule.add_systems((listen::update, beacon_query::QueryListener::recv));
    conn::GameConnection::register(&mut schedule);
    conn::RconConnection::register(&mut schedule);
    conn::MsmpConnection::register(&mut schedule);

    loop {
        schedule.run(&mut world);
    }
}
