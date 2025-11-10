use std::net::TcpListener;

use beacon_config::Config;
use beacon_net::{Listener, accept_tcp, update_listener};
use bevy_ecs::prelude::*;
use jsonrpc_core::IoHandler;

pub use crate::conn::MsmpConnection;

#[macro_use]
extern crate tracing;

mod conn;
mod rpc;

#[derive(Resource)]
pub struct MsmpListener {
    listener: TcpListener,
    io: IoHandler,
}

impl Listener for MsmpListener {
    fn new(config: &Config) -> std::io::Result<Self> {
        let listener = <TcpListener>::bind((config.msmp.ip, config.msmp.port))?;
        listener.set_nonblocking(true)?;

        let mut io = IoHandler::new();
        for method in inventory::iter::<rpc::RpcMethod> {
            method.add(&mut io);
        }

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
