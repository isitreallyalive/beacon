use std::{io, net::UdpSocket};

use beacon_config::Config;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct QueryListener {
    sock: UdpSocket,
}

impl QueryListener {
    pub fn new(config: &Config) -> io::Result<Self> {
        let sock = UdpSocket::bind((config.query.ip, config.query.port))?;
        sock.set_nonblocking(true)?;
        Ok(Self { sock })
    }

    pub fn recv(query: Option<Res<QueryListener>>) {
        if let Some(query) = query {
            let mut buf = [0u8; 1500]; // typical mtu
            match query.sock.recv_from(&mut buf) {
                Ok((size, addr)) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // no data available, non-blocking
                }
                Err(e) => {
                    eprintln!("Error receiving query packet: {}", e);
                }
            }
        }
    }
}

impl std::ops::Deref for QueryListener {
    type Target = UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.sock
    }
}
