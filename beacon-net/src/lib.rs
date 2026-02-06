//! # beacon-net
//!
//! This crate contains Minecraft protocol packet definitions and utilities for encoding/decoding them.

use beacon_codec::ProtocolState;
use bevy_ecs::prelude::*;

use crate::{conn::PacketQueue, server::ServerboundPacket};

#[macro_use]
extern crate derive_more;

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

/// Connection bundle.
pub mod conn;
/// Packet definitions and utilities.
pub mod packet;
/// Serverbound packets.
pub mod server;

mod prelude {
    pub use beacon_codec::types::*;
    pub use beacon_macros::packet;
}

/// Register packet handlers with the ECS.
pub fn ecs(schedule: &mut Schedule) {
    schedule.add_systems((packet::drain, handshake));
}

fn handshake(mut conns: Query<(&mut PacketQueue, &mut ProtocolState)>) -> Result<()> {
    for (mut queue, mut state) in conns.iter_mut() {
        if let Some(idx) = queue
            .iter()
            .position(|pkt| matches!(pkt, ServerboundPacket::Handshake(_)))
        {
            if let ServerboundPacket::Handshake(packet) = queue.remove(idx) {
                *state = packet.intent;
            }
        }
    }

    Ok(())
}
