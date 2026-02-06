use bevy_ecs::prelude::*;

use crate::{prelude::*, types::VarInt};

/// The protocol state of a connection, which determines which packets can be sent and received.
#[derive(Component, Clone, Copy, Debug, Display, Default, PartialEq, Eq)]
pub enum ProtocolState {
    /// Initial state after connection - doesn't really exist in the protocol.
    /// Used in place of [None](Option::None) to avoid using [Option].
    #[default]
    Handshake,
    /// Server List Ping.
    Status,
    /// Authentication, encryption, etc.
    Login,
    /// Transitions to the [Login](ProtocolState::Login) state, but indicates that the client connected
    /// due to a Transfer packet.
    Transfer,
    /// Share resource packs, registry data, etc.
    Configuration,
    /// Playing the game.
    Play,
}

impl Decode for ProtocolState {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        let state = VarInt::decode(read).await?;
        Ok(match *state {
            // you can only enter Status, Login, or Transfer from a Handshake packet - and that is
            // the only time a ProtocolState is decoded, so we don't need to worry about the other states here.
            1 => ProtocolState::Status,
            2 => ProtocolState::Login,
            3 => ProtocolState::Transfer,
            _ => return Err(DecodeError::InvalidProtocolState(state)),
        })
    }
}
