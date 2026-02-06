use std::net::SocketAddr;

use beacon_codec::ProtocolState;
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use crate::{packet::RawPacket, server::ServerboundPacket};

/// The client's socket address.
#[derive(Component)]
pub struct Address(SocketAddr);

/// The queue of packets to be processed.
#[derive(Component, Default, Deref, DerefMut)]
pub struct PacketQueue(Vec<ServerboundPacket>);

/// Sends raw packets to the connection for processing.
#[derive(Component, Deref)]
pub struct RawReceiver(Receiver<RawPacket>);

/// A connection to the server.
#[derive(Bundle)]
pub struct Connection {
    addr: Address,
    receiver: RawReceiver,
    state: ProtocolState,
    queue: PacketQueue,
}

impl Connection {
    /// Create a new connection with the given socket address.
    pub fn new(addr: SocketAddr) -> (Self, Sender<RawPacket>) {
        let (tx, rx) = crossbeam_channel::bounded(1024);
        let conn = Self {
            addr: Address(addr),
            receiver: RawReceiver(rx),
            state: ProtocolState::default(),
            queue: PacketQueue::default(),
        };
        (conn, tx)
    }
}
