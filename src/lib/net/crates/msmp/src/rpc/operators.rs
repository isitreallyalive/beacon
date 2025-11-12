//! Endpoints are accessible at `minecraft:operators`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Operators

use crate::{method, schema::Operator};

#[derive(Serialize)]
pub enum OperatorNotification {
    Added { player: Operator },
    Removed { player: Operator },
}

// todo: implement operator methods
// - /
// - /set
// - /add
// - /remove
// - /clear

method!("operators");
method!("operators/set");
method!("operators/add");
method!("operators/remove");
method!("operators/clear");
