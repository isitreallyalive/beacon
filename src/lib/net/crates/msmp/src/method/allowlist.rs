//! Endpoints are accessible at `minecraft:allowlist`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Allowlist

use crate::method;

method!("allowlist");
method!("allowlist/set");
method!("allowlist/add");
method!("allowlist/remove");
method!("allowlist/clear");
