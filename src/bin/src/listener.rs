use std::net::TcpListener;

use beacon_config::Config;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Listener(TcpListener);

impl std::ops::Deref for Listener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Listener {
    pub fn setup(world: &mut World) -> std::io::Result<()> {
        let port = world.resource::<Config>().port;
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        world.insert_resource(Listener(listener));
        Ok(())
    }
}
