use bevy_ecs::prelude::*;

use crate::listener::Listener;

mod listener;

fn main() -> Result<()> {
    let (mut world, mut schedule) = (World::new(), Schedule::default());
    world.insert_resource(Listener::bind(25565)?);

    loop {
        schedule.run(&mut world);
    }
}
