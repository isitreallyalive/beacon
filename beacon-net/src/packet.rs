use beacon_codec::{decode::*, types::VarInt};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};

/// A raw packet, before any processing is done.
#[derive(Debug)]
pub struct RawPacket {
    pub id: VarInt,
    pub data: Bytes,
}

impl Decode for RawPacket {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        // read header
        let length = VarInt::decode(read).await?;
        let id = VarInt::decode(read).await?;

        // read data
        let mut reader = read.take(*length as u64);
        let mut data = BytesMut::with_capacity(*length as usize);
        reader.read_buf(&mut data).await?;

        Ok(Self {
            id,
            data: data.freeze(),
        })
    }
}
