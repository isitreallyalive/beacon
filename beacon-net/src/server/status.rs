use beacon_config::Config;
use beacon_data::{LATEST_SUPPORTED_VERSION, PROTOCOL_VERSION};

use crate::{client::status::*, conn::PacketSender, prelude::*};

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Request>
#[packet(resource = "status_request", state = Status)]
#[derive(Debug)]
pub struct StatusRequest;

#[handler(StatusRequest)]
fn handle(config: Res<Config>, connections: Query<&mut PacketSender>) -> Result<()> {
    let payload = StatusResponsePayload {
        version: Version {
            name: LATEST_SUPPORTED_VERSION.to_string(),
            protocol: PROTOCOL_VERSION,
        },
        players: Players {
            max: config.server.max_players,
            // todo: change to actually online players, rather than connections
            online: connections.iter().count() as u32,
            sample: Vec::new()
        },
        description: Description { text: config.server.motd.clone() },
        favicon: None,
        secure_chat: false
    };
    let packet = StatusResponse::from(payload);
    let writer = connections.get(event.entity)?;

    writer.send(packet.blocking_raw()?)?;

    Ok(())
}

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Request>
#[packet(resource = "ping_request", state = Status)]
#[derive(Clone, Copy)]
pub struct PingRequest {
    payload: i64
}

#[handler(PingRequest)]
fn handle(mut query: Query<&mut PacketSender>) -> Result<()> {
    let writer = query.get_mut(event.entity)?;
    let packet = PongResponse::from(event.packet);
    writer.send(packet.blocking_raw()?)?;

    Ok(())
}