//! Endpoints are accessible at `minecraft:server`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Server

use crate::{method, schema::ServerState};

#[derive(Serialize)]
pub enum ServerNotification {
    Started,
    Stopping,
    Saving,
    Saved,
    Status { status: ServerState },
    Activity,
}

// todo: implement server methods
// - /status
// - /save
// - /stop
// - /system_message

method!("server/status");
method!("server/save");
method!("server/stop");
method!("server/system_message");
