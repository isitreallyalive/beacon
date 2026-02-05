//! See: <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#VarInt_and_VarLong>

use crate::prelude::*;

const SEGMENT: u8 = 0x7F;
const CONTINUE: u8 = 0x80;

/// Variable-length data encoding a [two's complement signed 32-bit integer](std::primitive::i32)
#[derive(Clone, Copy, Debug, Deref, Display, From, LowerHex, PartialEq, UpperHex)]
pub struct VarInt(i32);

impl VarInt {
    /// Returns the size of the [VarInt] when encoded.
    pub fn size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1, // ceil(log2(n + 1) / 7)
        }
    }
}

impl Decode for VarInt {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        let mut value = 0i32;
        let mut position = 0;
        for _ in 0..5 {
            let current = read.read_u8().await?;
            value |= ((current & SEGMENT) as i32) << position;
            if (current & CONTINUE) == 0 {
                return Ok(Self(value));
            }
            position += 7;
        }
        Err(DecodeError::VarIntTooBig)
    }
}

impl Encode for VarInt {
    async fn encode<W: AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), EncodeError> {
        let mut value = self.0 as u32;
        let segment = SEGMENT as u32;
        loop {
            if (value & !segment) == 0 {
                write.write_u8(value as u8).await?;
                return Ok(());
            }
            write.write_u8(((value & segment) as u8) | CONTINUE).await?;
            value >>= 7;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::VarInt;

    /// Taken from minecraft.wiki examples.
    ///
    /// See: https://minecraft.wiki/w/Java_Edition_protocol/Data_types#VarInt_and_VarLong
    const TEST_DATA: &[(VarInt, &[u8])] = &[
        (VarInt(0), &[0x00]),
        (VarInt(1), &[0x01]),
        (VarInt(2), &[0x02]),
        (VarInt(127), &[0x7F]),
        (VarInt(128), &[0x80, 0x01]),
        (VarInt(255), &[0xFF, 0x01]),
        (VarInt(25565), &[0xDD, 0xC7, 0x01]),
        (VarInt(2097151), &[0xFF, 0xFF, 0x7F]),
        (VarInt(2147483647), &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
        (VarInt(-1), &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        (VarInt(-2147483648), &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];

    #[tokio::test]
    async fn test_decode() -> Result<(), DecodeError> {
        for (expected, data) in TEST_DATA {
            let got = VarInt::decode(&mut &data[..]).await?;
            assert_eq!(got, *expected);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_encode() -> Result<(), EncodeError> {
        for (data, expected) in TEST_DATA {
            let mut buf = Vec::new();
            data.encode(&mut buf).await?;
            assert_eq!(&buf[..], *expected);
        }
        Ok(())
    }
}
