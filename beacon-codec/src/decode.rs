use miette::Diagnostic;
use thiserror::Error;

use crate::prelude::*;

/// Error that can occur during decoding.
#[derive(Debug, Error, Diagnostic)]
pub enum DecodeError {
    /// A miscellaneous I/O error occurred while reading from the client.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A VarInt was too big (more than 5 bytes).
    #[error("VarInt is too big")]
    #[diagnostic(help("VarInts must be at most 5 bytes long"))]
    VarIntTooBig,
}

/// Trait for types that can be decoded from a Minecraft client.
#[allow(async_fn_in_trait)]
pub trait Decode: Sized {
    /// Decode the type by reading from the provided reader.
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError>;
}
