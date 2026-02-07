use serde::Serialize;

use crate::prelude::*;

/// Helper type to encode data as a JSON string.
#[derive(From)]
pub struct Json<T: Serialize + Sync>(pub T);

impl<T: Serialize + Sync> Encode for Json<T> {
    async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError> {
        let json = serde_json::to_string(&self.0)?;
        json.encode(write).await
    }
}
