use bevy_ecs::prelude::*;

/// The player's identity, containing their username and UUID.
#[derive(Debug, Component)]
pub struct PlayerIdentity {
    /// The player's username.
    pub name: String,
    /// The player's UUID.
    pub uuid: u128,
}
