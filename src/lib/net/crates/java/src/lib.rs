use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, update_listener};
use bevy_ecs::prelude::*;

mod conn;
pub use conn::JavaConnection;

#[derive(Resource)]
pub struct JavaListener(TcpListener);

impl Listener for JavaListener {
    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.server.ip, config.server.port))?;
        listener.set_nonblocking(true)?;
        Ok(JavaListener(listener))
    }

    update_listener!(JavaListener: server);
}

impl std::ops::Deref for JavaListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
