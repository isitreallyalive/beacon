//! # beacon-net
//!
//! This crate contains Minecraft protocol packet definitions and utilities for encoding/decoding them.

use beacon_codec::{ProtocolState, decode::Decode};
use bevy_ecs::prelude::*;
use futures::executor::block_on;

use crate::server::*;
use crate::{
    conn::{Despawn, PacketReceiver},
    packet::PacketData,
};

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate tracing;

macro_rules! dispatch {
    ($packet:ident, $raw:expr, $entity:expr, $commands:expr) => {
        match block_on($packet::decode(&mut $raw.data.as_ref())) {
            Ok(packet) => $commands.trigger(packet.event($entity)),
            Err(err) => {
                $commands.entity($entity).despawn();
                error!(%err, "failed to decode packet");
            }
        }
    };
}

/// Register all networking systems with the ECS.
pub fn ecs(schedule: &mut Schedule) {
    schedule.add_systems((listen, despawn));
}

/// Despawn closed connections.
fn despawn(mut commands: Commands, query: Query<(Entity, &Despawn)>) {
    for (entity, cancel) in query.iter() {
        if cancel.is_cancelled() {
            commands.entity(entity).despawn();
        }
    }
}

macro_rules! packets {
    (
        $(
            $packet:ident
        ),*
    ) => {
        /// Listen for incoming packets and trigger events for them.
        fn listen(
            mut query: Query<(Entity, &ProtocolState, &PacketReceiver)>,
            mut commands: Commands,
        ) -> Result<()> {
            // todo: take turns reading packets
            for (entity, state, rx) in query.iter_mut() {
                while let Ok(packet) = rx.try_recv() {
                    match (state, packet.id) {
                        (&Handshake::STATE, Handshake::ID) => {
                            // handshake must be processed before any other subsequent packets.
                            dispatch!(Handshake, packet, entity, commands);
                            break;
                        },
                        $(
                            (&$packet::STATE, $packet::ID) => {
                                // todo: close connection on error
                                dispatch!($packet, packet, entity, commands);
                            },
                        )*
                        _ => {
                            // unknown packet
                            warn!(id = %packet.id, state = %state, "unknown packet");
                        }
                    }
                }
            }

            Ok(())
        }

        /// Add handlers for all packets to an entity.
        fn observe_packets(entity: &mut EntityWorldMut) {
            entity.observe(Handshake::handle);
            $(
                entity.observe($packet::handle);
            )*
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
mod client {
    #[allow(missing_docs)]
    pub mod status;
}
/// Serverbound packets.
mod server {
    import!(handshake, status);
}
/// Connection bundle.
pub mod conn;
/// Packet definitions and utilities.
pub mod packet;

mod prelude {
    pub use beacon_codec::types::*;
    pub use beacon_macros::{handler, packet};
    pub use bevy_ecs::prelude::*;
}
