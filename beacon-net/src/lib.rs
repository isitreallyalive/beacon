//! # beacon-net
//!
//! This crate contains Minecraft protocol packet definitions and utilities for encoding/decoding them.

use beacon_codec::ProtocolState;
use bevy_ecs::{prelude::*, system::SystemState, world::CommandQueue};

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate tracing;

macro_rules! packets {
    (
        $(
            $packet:ident
        ),*
    ) => {
        use beacon_codec::decode::Decode;
        use futures::executor::block_on;

        use crate::packet::{RawPacket, PacketData};
        use crate::server::*;

        /// Listen for incoming packets and trigger events for them.
        pub fn listen(
            mut query: Query<(Entity, &ProtocolState, &RawReceiver)>,
            mut commands: Commands,
        ) -> Result<()> {
            // todo: take turns reading packets
            for (entity, state, rx) in query.iter_mut() {
                while let Ok(packet) = rx.try_recv() {
                    match (state, packet.id) {
                        (&Handshake::STATE, Handshake::ID) => {
                            // handshake must be processed before any other subsequent packets.
                            let event = block_on(Handshake::decode(&mut packet.data.as_ref()))?.event(entity);
                            commands.trigger(event);
                            break;
                        },
                        $(
                            (&$packet::STATE, $packet::ID) => {
                                // todo: close connection on error
                                let event = block_on($packet::decode(&mut packet.data.as_ref()))?.event(entity);
                                commands.trigger(event);
                            },
                        )*
                        // unknown packet
                        _ => unimplemented!()
                    }
                }
            }

            Ok(())
        }

        impl Connection {
            /// Spawn a new connection and return a sender for raw packets.
            pub fn spawn(world: &mut World) -> crossbeam_channel::Sender<RawPacket> {
                let (tx, rx) = crossbeam_channel::bounded(1024);

                world.spawn(Self {
                    receiver: RawReceiver(rx),
                    state: ProtocolState::default(),
                })
                    .observe(Handshake::handle)
                    $(.observe($packet::handle))*;

                tx
            }
        }
    };
}

packets! {
    StatusRequest
}

/// Re-export everything from a module.
macro_rules! import {
    ($($name:ident),*) => {
        $(
            mod $name;
            pub use $name::*;
        )*
    };
}

/// Clientbound packets.
pub mod client {}
/// Serverbound packets.
pub mod server {
    import!(handshake, status);
}
/// Packet definitions and utilities.
pub mod packet;

mod prelude {
    pub use beacon_codec::types::*;
    pub use beacon_macros::{handler, packet};
    pub use bevy_ecs::prelude::*;
}

/// Sends raw packets to the connection for processing.
#[derive(Component, Deref)]
pub struct RawReceiver(crossbeam_channel::Receiver<RawPacket>);

/// A connection to the server.
#[derive(Bundle)]
pub struct Connection {
    receiver: RawReceiver,
    state: ProtocolState,
}
