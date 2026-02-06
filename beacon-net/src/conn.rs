use std::net::SocketAddr;

use beacon_codec::ProtocolState;
use bevy_ecs::prelude::*;

/// The client's socket address.
#[derive(Component)]
pub struct Address(SocketAddr);

/// A connection to the server.
#[derive(Bundle)]
pub struct Connection {
    addr: Address,
    state: ProtocolState,
}

impl From<SocketAddr> for Connection {
    fn from(addr: SocketAddr) -> Self {
        Self {
            addr: Address(addr),
            state: ProtocolState::default(),
        }
    }
}
