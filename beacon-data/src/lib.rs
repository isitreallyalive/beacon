//! # beacon-data
//!
//! Data used by the beacon Minecraft server.

#[macro_use]
extern crate derive_more;

/// A Minecraft version number.
#[derive(Clone, Copy, Display)]
#[display("1.{}.{}", self.0, self.1)]
pub struct Version(u8, u8);

/// The current version of beacon.
pub const BEACON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the Minecraft protocol that beacon uses.
///
/// See: <https://minecraft.wiki/w/Protocol_version>
pub const PROTOCOL_VERSION: u16 = 774;

/// The versions of Minecraft that beacon supports.
pub const SUPPORTED_VERSIONS: &[Version] = &[Version(21, 11)];

/// The latest version of Minecraft that beacon supports.
pub const LATEST_SUPPORTED_VERSION: Version = SUPPORTED_VERSIONS[SUPPORTED_VERSIONS.len() - 1];
