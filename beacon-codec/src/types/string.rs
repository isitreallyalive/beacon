//! See: <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Type:String>

use crate::{prelude::*, types::VarInt};

impl Decode for String {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        let length = VarInt::decode(read).await?.0 as usize;
        let mut string_bytes = vec![0u8; length];
        read.read_exact(&mut string_bytes).await?;
        let string = String::from_utf8(string_bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        Ok(string)
    }
}

impl Encode for String {
    async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError> {
        let string_bytes = self.as_bytes();
        let length = VarInt(string_bytes.len() as i32);
        length.encode(write).await?;
        write.write_all(string_bytes).await?;

        Ok(())
    }
}
