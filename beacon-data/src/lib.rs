/// The current version of beacon.
pub const BEACON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Minecraft versions supported by beacon.
pub const SUPPORTED_VERSIONS: [&str; 2] = ["1.21.9", "1.21.10"];

/// The latest Minecraft version supported by beacon.
pub const LATEST_SUPPORTED: &str = SUPPORTED_VERSIONS[SUPPORTED_VERSIONS.len() - 1];

/// The protocol version corresponding with the Minecraft version that beacon targets.
///
/// See: https://minecraft.wiki/w/Protocol_version
pub const PROTOCOL_VERSION: u16 = 773;
