//! Endpoints are accessible at `minecraft:ip_bans`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#IP_Bans

use crate::{method, schema::IPBan};

#[derive(Serialize)]
pub enum IPBanNotification {
    Added { player: IPBan },
    Removed { player: String },
}

// todo: implement ip_ban methods
// - /
// - /set
// - /add
// - /remove
// - /clear

method!("ip_bans");
method!("ip_bans/set");
method!("ip_bans/add");
method!("ip_bans/remove");
method!("ip_bans/clear");
