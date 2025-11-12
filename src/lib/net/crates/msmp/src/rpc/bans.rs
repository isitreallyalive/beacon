//! Endpoints are accessible at `minecraft:bans`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Bans

use crate::{method, schema::Operator};

#[derive(Serialize)]
pub enum BanNotification {
    Added { player: Operator },
    Removed { player: Operator },
}

// todo: implement ban methods
// - /
// - /set
// - /add
// - /remove
// - /clear

method!("bans");
method!("bans/set");
method!("bans/add");
method!("bans/remove");
method!("bans/clear");
