use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, update_listener};
use bevy_ecs::prelude::*;

mod conn;
pub use conn::RconConnection;

#[derive(Resource)]
pub struct RconListener(TcpListener);

impl Listener for RconListener {
    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.msmp.ip, config.msmp.port))?;
        listener.set_nonblocking(true)?;
        Ok(RconListener(listener))
    }

    update_listener!(RconListener: rcon);
}

impl std::ops::Deref for RconListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
