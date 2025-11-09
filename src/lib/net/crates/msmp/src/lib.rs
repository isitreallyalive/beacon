use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, accept_tcp, update_listener};
use bevy_ecs::prelude::*;

#[macro_use]
extern crate tracing;

mod conn;
pub use conn::MsmpConnection;
use jsonrpc_core::IoHandler;

#[derive(Resource)]
pub struct MsmpListener {
    listener: TcpListener,
    io: IoHandler,
}

impl Listener for MsmpListener {
    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.msmp.ip, config.msmp.port))?;
        listener.set_nonblocking(true)?;

        let io = IoHandler::new();

        Ok(MsmpListener { listener, io })
    }

    update_listener!(msmp);

    accept_tcp!(|conn, addr, commands| {
        let ws = tungstenite::accept(conn)?;
        commands.spawn(MsmpConnection { ws, addr });
    });
}

impl std::ops::Deref for MsmpListener {
    type Target = TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}
