//! Endpoints are accessible at `minecraft:allowlist`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Allowlist

use crate::{method, schema::Player};

#[derive(Serialize)]
pub enum AllowlistNotification {
    Added { player: Player },
    Removed { player: Player },
}

// todo: implement allowlist methods
// - /
// - /set
// - /add
// - /remove
// - /clear

method!("allowlist");
method!("allowlist/set");
method!("allowlist/add");
method!("allowlist/remove");
method!("allowlist/clear");
