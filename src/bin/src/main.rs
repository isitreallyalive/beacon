use beacon_config::Config;
use beacon_net::{Connection, Listener};
use bevy_ecs::prelude::*;

fn main() -> Result<()> {
    // setup logging
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // start bevy
    let (mut world, mut schedule) = (World::new(), Schedule::default());
    let config = Config::setup(&mut world, &mut schedule, "beacon.toml")?;

    // setup listeners
    beacon_java::JavaListener::register(&mut world, &mut schedule, &config)?;
    beacon_java::JavaConnection::register(&mut schedule);
    beacon_msmp::MsmpListener::register(&mut world, &mut schedule, &config)?;
    beacon_msmp::MsmpConnection::register(&mut schedule);
    beacon_query::QueryListener::register(&mut world, &mut schedule, &config)?;
    beacon_rcon::RconListener::register(&mut world, &mut schedule, &config)?;
    beacon_rcon::RconConnection::register(&mut schedule);

    loop {
        schedule.run(&mut world);
    }
}
