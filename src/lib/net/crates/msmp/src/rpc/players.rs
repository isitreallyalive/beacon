//! Endpoints are accessible at `minecraft:players`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Players

use crate::{method, schema::Player};

#[derive(Serialize)]
pub enum PlayerNotification {
    Joined { player: Player },
    Left { player: Player },
}

// todo: implement player methods
// - /
// - /kick

method!("players");
method!("players/kick");
