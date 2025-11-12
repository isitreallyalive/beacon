//! Endpoints are accessible at `minecraft:ip_bans`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#IP_Bans

use crate::method;

method!("ip_bans");
method!("ip_bans/set");
method!("ip_bans/add");
method!("ip_bans/remove");
method!("ip_bans/clear");
