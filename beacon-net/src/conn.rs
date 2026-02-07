use beacon_codec::ProtocolState;
use bevy_ecs::prelude::*;
use flume::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::{observe_packets, packet::RawPacket};

/// Receiver for incoming packets.
#[derive(Component, Deref)]
pub struct PacketReceiver(Receiver<RawPacket>);

/// Sender for outgoing packets.
#[derive(Component, Deref)]
pub struct PacketSender(Sender<RawPacket>);

/// A cancellation token used to despawn a connection when it's closed.
#[derive(Component, Deref)]
pub struct Despawn(CancellationToken);

/// A connection to the server.
#[derive(Bundle)]
#[non_exhaustive]
pub struct Connection {
    receiver: PacketReceiver,
    sender: PacketSender,
    state: ProtocolState,
    despawn: Despawn,
}

impl Connection {
    /// Spawn a new connection and add it to the world.
    pub fn spawn(
        world: &mut World,
    ) -> (
        flume::Sender<RawPacket>,
        flume::Receiver<RawPacket>,
        CancellationToken,
    ) {
        // open channels
        let (in_tx, in_rx) = flume::bounded(1024);
        let (out_tx, out_rx) = flume::bounded(1024);
        let token = CancellationToken::new();

        // spawn entity
        let mut entity = world.spawn(Self {
            receiver: PacketReceiver(in_rx),
            sender: PacketSender(out_tx),
            state: ProtocolState::default(),
            despawn: Despawn(token.clone()),
        });
        observe_packets(&mut entity);

        (in_tx, out_rx, token)
    }
}
