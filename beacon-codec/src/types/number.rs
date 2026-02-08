/// Various numeric types.
use pastey::paste;

use crate::prelude::*;

macro_rules! num {
    (
        $(
            $ty:ty
        ),+
    ) => {
        $(
            impl Decode for $ty {
                async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
                    paste! { read.[<read_ $ty>]().await.map_err(DecodeError::from) }
                }
            }

            impl Encode for $ty {
                async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError> {
                    paste! { write.[<write_ $ty>](*self).await.map_err(EncodeError::from) }
                }
            }
        )+
    };
}

num!(u16, u128, i64);
