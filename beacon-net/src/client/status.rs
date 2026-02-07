use beacon_data::{LATEST_SUPPORTED_VERSION, PROTOCOL_VERSION};
use serde::Serialize;

use crate::prelude::*;

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

#[derive(Debug, Serialize)]
pub struct Version {
    #[serde(default = "Old")]
    name: String,
    protocol: u16,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            name: LATEST_SUPPORTED_VERSION.to_string(),
            protocol: PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Players {
    pub max: u32,
    pub online: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sample: Vec<SamplePlayer>,
}

#[derive(Debug, Serialize)]
pub struct SamplePlayer {
    pub name: String,
    pub id: String,
}

// todo: replace with text component
// see: https://minecraft.wiki/w/Text_component_format
#[derive(Debug, Serialize)]
pub struct Description {
    pub text: String,
}

// todo: cache response
/// See: https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Response
#[derive(Debug, Serialize)]
pub struct StatusResponsePayload {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub secure_chat: bool,
}
