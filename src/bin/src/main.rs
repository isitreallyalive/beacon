use beacon_config::Config;
use bevy_ecs::prelude::*;

use crate::listener::Listener;

mod listener;

fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (mut world, mut schedule) = (World::new(), Schedule::default());
    Config::setup(&mut world, &mut schedule, "beacon.toml")?;
    Listener::setup(&mut world)?;

    loop {
        schedule.run(&mut world);
    }
}
