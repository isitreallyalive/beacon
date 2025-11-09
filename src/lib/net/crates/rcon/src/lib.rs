use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, accept_tcp, update_listener};
use bevy_ecs::prelude::*;

#[macro_use]
extern crate tracing;

mod conn;
pub use conn::RconConnection;

#[derive(Resource)]
pub struct RconListener(TcpListener);

impl Listener for RconListener {
    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.rcon.ip, config.rcon.port))?;
        listener.set_nonblocking(true)?;
        Ok(RconListener(listener))
    }

    accept_tcp!(RconConnection);
    update_listener!(rcon);
}

impl std::ops::Deref for RconListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
