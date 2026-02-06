use bevy_ecs::prelude::*;

import!(handshake, status);

/// All serverbound packets.
#[derive(Message, Debug)]
#[allow(missing_docs)]
pub enum ServerboundPacket {
    Handshake(Handshake),
    StatusRequest(StatusRequest),
}
