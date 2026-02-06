//! # beacon-codec
//!
//! This crate contains...
//! - Traits for encoding and decoding data to and from a Minecraft client.
//! - Implementations of these traits for various Minecraft data types (i.e. [VarInt])

pub use crate::state::ProtocolState;

#[macro_use]
extern crate derive_more;

/// Encoding trait.
pub mod encode;

/// Decoding trait.
pub mod decode;

/// Common types used by the Minecraft protocol.
pub mod types {
    pub use varint::VarInt;

    mod number;
    mod string;
    mod varint;
}

mod state;

mod prelude {
    pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    pub use crate::{decode::*, encode::*};
}
