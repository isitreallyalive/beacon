use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, update_listener};
use bevy_ecs::prelude::*;

#[macro_use]
extern crate tracing;

mod conn;
pub use conn::MsmpConnection;

#[derive(Resource)]
pub struct MsmpListener(TcpListener);

impl Listener for MsmpListener {
    const NAME: &str = "MSMP";

    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.msmp.ip, config.msmp.port))?;
        listener.set_nonblocking(true)?;
        Ok(MsmpListener(listener))
    }

    update_listener!(msmp);

    fn accept(listener: Option<Res<Self>>, commands: Commands) {}
}

impl std::ops::Deref for MsmpListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
