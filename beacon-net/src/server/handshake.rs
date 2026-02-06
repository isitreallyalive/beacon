use beacon_codec::{ProtocolState};

use crate::prelude::*;

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Handshake>
#[packet(resource = "intention", state = Handshake)]
#[derive(Debug)]
pub struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: u16,
    intent: ProtocolState
}
