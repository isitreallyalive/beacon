/// The current version of beacon.
pub const BEACON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The protocol version corresponding with the Minecraft version that beacon targets.
/// This is currently Minecraft **1.21.9** and **1.21.10**.
///
/// See: https://minecraft.wiki/w/Protocol_version
pub const PROTOCOL_VERSION: u16 = 773;

/// The (latest) supported Minecraft version by beacon.
pub const SUPPORTED_VERSION: &str = "1.21.10";
