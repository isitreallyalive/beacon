use crate::{player::PlayerIdentity, prelude::*};

#[server(resource = "hello", state = Login)]
pub struct LoginStart {
    name: String,
    uuid: u128
}

#[handler(LoginStart)]
fn handle(mut commands: Commands) {
    let id = PlayerIdentity { name: event.packet.name.clone(), uuid: event.packet.uuid };
    commands.entity(event.entity).insert(id);

    // todo: respond with EncryptionRequest
}