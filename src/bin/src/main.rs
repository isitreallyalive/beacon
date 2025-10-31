use beacon_config::Config;
use bevy_ecs::prelude::*;

use crate::listener::Listener;

mod listener;

fn main() -> Result<()> {
    let config = Config::read("beacon.toml")?;
    let (mut world, mut schedule) = (World::new(), Schedule::default());
    world.insert_resource(Listener::bind(config.port)?);

    loop {
        schedule.run(&mut world);
    }
}
