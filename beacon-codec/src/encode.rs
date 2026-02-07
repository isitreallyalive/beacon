use miette::Diagnostic;
use thiserror::Error;

use crate::prelude::*;

/// Error that can occur during encoding.
#[derive(Debug, Error, Diagnostic)]
pub enum EncodeError {
    /// A miscellaneous I/O error occurred while writing to the client.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// An error occurred when serializing a value to JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Trait for types that can be encoded and sent to a Minecraft client.
#[allow(async_fn_in_trait)]
pub trait Encode {
    /// Encode the type and write it to the provided writer.
    async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError>;
}
