use serde::Serialize;

use crate::{prelude::*, server::PingRequest};

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Response>
#[packet(resource = "status_response", state = Status)]
pub struct StatusResponse {
    json: Json<StatusResponsePayload>,
}

impl From<StatusResponsePayload> for StatusResponse {
    fn from(payload: StatusResponsePayload) -> Self {
        Self {
            json: Json(payload),
        }
    }
}

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Response>
#[packet(resource = "pong_response", state = Status)]
pub struct PongResponse {
    payload: i64,
}

impl From<PingRequest> for PongResponse {
    fn from(request: PingRequest) -> Self {
        Self {
            payload: request.payload,
        }
    }
}

/// The version of Minecraft the server supports.
#[derive(Debug, Serialize)]
pub struct Version {
    /// The name of the version, e.g. "1.21.1"
    pub name: String,
    /// The protocol version, e.g. 774
    pub protocol: u16,
}

/// Information about the players on the server.
#[derive(Debug, Serialize)]
pub struct Players {
    /// How many players can the server support at once.
    pub max: u32,
    /// How many players are currently online.
    pub online: u32,
    /// A sample of the players currently online.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sample: Vec<SamplePlayer>,
}

/// A player who is currently online on the server.  
#[derive(Debug, Serialize)]
pub struct SamplePlayer {
    /// The player's username.
    pub name: String,
    /// The player's UUID.
    pub id: String,
}

// todo: replace with text component
// see: https://minecraft.wiki/w/Text_component_format

/// A text component containing the server's MOTD.
#[derive(Debug, Serialize)]
pub struct Description {
    /// The text of the MOTD.
    pub text: String,
}

// todo: cache response
/// See: https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Response
#[derive(Debug, Serialize)]
pub struct StatusResponsePayload {
    /// The version of Minecraft the server supports.
    pub version: Version,
    /// Information about the players on the server.
    pub players: Players,
    /// The server's MOTD.
    pub description: Description,
    /// The server's favicon, if it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// Whether the server enforces secure chat.
    #[serde(rename = "enforcesSecureChat")]
    pub secure_chat: bool,
}
