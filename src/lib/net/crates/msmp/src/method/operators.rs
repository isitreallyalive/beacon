//! Endpoints are accessible at `minecraft:operators`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Operators

use crate::method;

method!("operators");
method!("operators/set");
method!("operators/add");
method!("operators/remove");
method!("operators/clear");
