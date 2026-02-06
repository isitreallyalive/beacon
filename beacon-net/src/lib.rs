//! # beacon-net
//!
//! This crate contains Minecraft protocol packet definitions and utilities for encoding/decoding them.

/// Re-export everything from a module.
macro_rules! import {
    ($($name:ident),*) => {
        $(
            mod $name;
            pub use $name::*;
        )*
    };
}

/// Clientbound packets.
pub mod client {}

/// Serverbound packets.
pub mod server {
    import!(handshake);
}

/// Packet definitions and utilities.
pub mod packet;

mod prelude {
    pub use beacon_codec::types::*;
    pub use beacon_macros::packet;
}
