//! Endpoints are accessible at `minecraft:server`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Server

use crate::method;

method!("server/status");
method!("server/save");
method!("server/stop");
method!("server/system_message");
