use beacon_config::Config;
use bevy_ecs::prelude::*;

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
    world.insert_resource(net::Game::new(&config)?);
    schedule.add_systems(net::update);

    loop {
        schedule.run(&mut world);
    }
}
