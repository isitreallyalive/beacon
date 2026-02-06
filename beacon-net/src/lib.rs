//! # beacon-net
//!
//! This crate contains Minecraft protocol packet definitions and utilities for encoding/decoding them.
/// Re-export everything from a module.

#[macro_use]
extern crate derive_more;

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

/// Connection bundle.
pub mod conn;
/// Packet definitions and utilities.
pub mod packet;
/// Serverbound packets.
pub mod server;

mod prelude {
    pub use beacon_codec::types::*;
    pub use beacon_macros::packet;
}
