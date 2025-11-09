use std::io::{self, Cursor};

use deku::{ctx::Endian, no_std_io, prelude::*};

#[derive(Debug, DekuRead)]
#[cfg_attr(test, derive(DekuWrite))]
#[deku(endian = "big", id_type = "u8", magic = b"\xFE\xFD")]
pub enum QueryRequest {
    #[deku(id = "0x00")]
    Stat(StatRequest),
    #[deku(id = "0x09")]
    Handshake,
}

#[derive(Debug)]
pub struct StatRequest {
    pub session_id: i32,
    pub challenge_token: i32,
    pub full: bool,
}

impl DekuReader<'_, Endian> for StatRequest {
    fn from_reader_with_ctx<R: no_std_io::Read + io::Seek>(
        reader: &mut Reader<R>,
        ctx: Endian,
    ) -> Result<Self, DekuError> {
        let session_id = i32::from_reader_with_ctx(reader, ctx)?;
        let challenge_token = i32::from_reader_with_ctx(reader, ctx)?;
        Ok(StatRequest {
            session_id,
            challenge_token,
            full: !reader.end(),
        })
    }
}

#[cfg(test)]
impl DekuWriter<Endian> for StatRequest {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        self.session_id.to_writer(writer, ctx)?;
        self.challenge_token.to_writer(writer, ctx)?;
        if self.full {
            [0u8; 4].to_writer(writer, ctx)?; // padding
        }
        Ok(())
    }
}

impl QueryRequest {
    /// Maximum size of a c2s packet (full stat)
    // 2 (magic) + 1 (id) + 4 (session id) + 4 (challenge token) + 4 (padding)
    pub const MAX_SIZE: usize = 2 + 1 + 4 + 4 + 4;
}
