use beacon_config::Config;

use crate::{RawSender, client::status::*, prelude::*};

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Request>
#[packet(resource = "status_request", state = Status)]
#[derive(Debug)]
pub struct StatusRequest;

#[handler(StatusRequest)]
fn handle(config: Res<Config>, connections: Query<&mut RawSender>) -> Result<()> {
    let payload = StatusResponsePayload {
        version: Version::default(),
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
    let mut writer = connections.get(event.entity)?;
    packet.blocking_raw().map(|p| writer.send(p))?;

    Ok(())
}