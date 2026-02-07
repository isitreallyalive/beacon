use beacon_config::{Config, FAVICON};
use beacon_data::{LATEST_SUPPORTED_VERSION, PROTOCOL_VERSION};

use crate::{client::status::*, conn::{Despawn, PacketSender}, prelude::*};

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Request>
#[packet(resource = "status_request", state = Status)]
#[derive(Debug)]
pub struct StatusRequest;

#[handler(StatusRequest)]
fn handle(config: Res<Config>, query: Query<(&Despawn, &PacketSender)>) -> Result<()> {
    // do not respond if status is disabled
    let (despawn, sender) = query.get(event.entity)?;
    if !config.server.status {
        despawn.cancel();
        return Ok(());
     }

    let payload = StatusResponsePayload {
        version: Version {
            name: LATEST_SUPPORTED_VERSION.to_string(),
            protocol: PROTOCOL_VERSION,
        },
        players: Players {
            max: config.server.max_players,
            // todo: change to actually online players, rather than connections
            online: query.iter().count() as u32,
            sample: Vec::new()
        },
        description: Description { text: config.server.motd.clone() },
        favicon: FAVICON.read().ok().and_then(|f| f.clone()),
        secure_chat: false
    };
    let packet = StatusResponse::from(payload);

    sender.send(packet.blocking_raw()?)?;

    Ok(())
}

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Request>
#[packet(resource = "ping_request", state = Status)]
#[derive(Clone, Copy)]
pub struct PingRequest {
    payload: i64
}

#[handler(PingRequest)]
fn handle(config: Res<Config>, query: Query<(&Despawn, &PacketSender)>) -> Result<()> {
    // do not respond if status is disabled
    let (despawn, sender) = query.get(event.entity)?;
    if !config.server.status {
        despawn.cancel();
        return Ok(());
     }

    let packet = PongResponse::from(event.packet);
    sender.send(packet.blocking_raw()?)?;

    Ok(())
}