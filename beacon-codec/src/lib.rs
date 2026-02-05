//! # beacon-codec
//!
//! This crate contains...
//! - Traits for encoding and decoding data to and from a Minecraft client.
//! - Implementations of these traits for various Minecraft data types (i.e. [VarInt])

#[macro_use]
extern crate derive_more;

/// Encoding trait.
pub mod encode;

/// Decoding trait.
pub mod decode;

/// Common types used by the Minecraft protocol.
pub mod types {
    pub use varint::VarInt;

    mod varint;
}

mod prelude {
    pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    pub use crate::{decode::*, encode::*};
}
