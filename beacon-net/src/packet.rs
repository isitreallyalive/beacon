use beacon_codec::{
    ProtocolState,
    decode::*,
    encode::{Encode, EncodeError},
    types::VarInt,
};
use bevy_ecs::prelude::*;
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// A raw packet, before any processing is done.
#[derive(Debug)]
pub struct RawPacket {
    pub(crate) id: VarInt,
    pub(crate) data: Bytes,
}

impl Decode for RawPacket {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        // read header
        let length = VarInt::decode(read).await?;
        let id = VarInt::decode(read).await?;

        // read data
        let remaining = *length as usize - id.size();
        let mut reader = read.take(remaining as u64);
        let mut data = BytesMut::with_capacity(remaining);
        reader.read_buf(&mut data).await?;

        Ok(Self {
            id,
            data: data.freeze(),
        })
    }
}

impl Encode for RawPacket {
    async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError> {
        let mut buf = Vec::new();
        self.id.encode(&mut buf).await?;
        buf.extend_from_slice(&self.data);

        let length = VarInt(buf.len() as i32);
        length.encode(write).await?;
        write.write_all(&buf).await?;

        Ok(())
    }
}

/// Trait containing packet metadata.
pub trait PacketData {
    /// The packet ID.
    const ID: VarInt;

    /// The protocol state this packet belongs to.
    const STATE: ProtocolState;
}
