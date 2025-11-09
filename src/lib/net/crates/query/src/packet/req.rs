use std::io;

use deku::{ctx::Endian, prelude::*};

#[derive(Debug, DekuRead)]
#[cfg_attr(test, derive(DekuWrite))]
#[deku(endian = "big", id_type = "u8", magic = b"\xFE\xFD")]
pub enum QueryRequest {
    #[deku(id = "0x00")]
    Stat(StatRequest),
    #[deku(id = "0x09")]
    Handshake { session_id: i32 },
}

impl QueryRequest {
    pub const MAX_SIZE: usize = 15; // magic(2) + id(1) + session(4) + token(4) + padding(4)
}

#[derive(Debug)]
pub struct StatRequest {
    pub session_id: i32,
    pub challenge_token: i32,
    pub full: bool,
}

impl DekuReader<'_, Endian> for StatRequest {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
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
            [0u8; 4].to_writer(writer, ctx)?;
        }
        Ok(())
    }
}
