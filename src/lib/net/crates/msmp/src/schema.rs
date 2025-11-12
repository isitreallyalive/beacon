//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Schemas

use std::borrow::Cow;

// todo: implement schemas
// - Untyped Game Rule
// - Incoming IP Ban
// - System Message
// - Kick Player

#[derive(Serialize, Deserialize)]
pub struct IPBan {
    reason: String,
    expires: String,
    ip: String,
    source: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameRuleType {
    Integer,
    Boolean,
}

#[derive(Serialize, Deserialize)]
pub struct TypedGameRule {
    r#type: GameRuleType,
    value: String,
    key: String,
}

// todo: implement schemas
// - User Ban
// - Message

#[derive(Serialize, Deserialize)]
pub struct Version {
    protocol: u32,
    name: Cow<'static, str>,
}

impl Version {
    /// The current server version.
    const CURRENT: Self = Self {
        protocol: beacon_config::PROTOCOL_VERSION,
        name: Cow::Borrowed(beacon_config::VERSION),
    };
}

#[derive(Serialize, Deserialize)]
pub struct ServerState {
    players: Vec<Player>,
    started: bool,
    version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Operator {
    #[serde(rename = "permissionLevel")]
    permission_evel: u8,
    #[serde(rename = "bypassesPlayerLimit")]
    bypasses_player_limit: bool,
    player: Player,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    id: String,
}
