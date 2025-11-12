//! Endpoints are accessible at `minecraft:bans`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Bans

use crate::method;

method!("bans");
method!("bans/set");
method!("bans/add");
method!("bans/remove");
method!("bans/clear");
