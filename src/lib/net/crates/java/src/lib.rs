use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, accept_tcp, update_listener};
use bevy_ecs::prelude::*;

#[macro_use]
extern crate tracing;

mod conn;
pub use conn::JavaConnection;

#[derive(Resource)]
pub struct JavaListener(TcpListener);

impl Listener for JavaListener {
    const NAME: &str = "Server";

    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.server.ip, config.server.port))?;
        listener.set_nonblocking(true)?;
        Ok(JavaListener(listener))
    }

    update_listener!(server);
    accept_tcp!(JavaConnection);
}

impl std::ops::Deref for JavaListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
