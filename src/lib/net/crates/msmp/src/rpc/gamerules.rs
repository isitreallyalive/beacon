//! Endpoints are accessible at `minecraft:gamerules`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Gamerules

use crate::{method, schema::TypedGameRule};

#[derive(Serialize)]
pub enum GameruleNotification {
    Updated { gamerule: TypedGameRule },
}

// todo: implement gamerules methods
// - /
// - /update

method!("gamerules");
method!("gamerules/update");
