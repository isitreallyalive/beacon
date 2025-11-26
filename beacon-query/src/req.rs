use std::io;

use deku::{ctx::Endian, prelude::*};

/// Maximum size of a serialized QueryRequest in bytes. Happens when FullStat is requested.
///
/// MAX_SIZE = magic(2) + id(1) + session(4) + token(4) + padding(4)
pub const MAX_SIZE: usize = 2 + 1 + 4 + 4 + 4;

#[derive(Debug, DekuRead)]
#[deku(endian = "big", id_type = "u8", magic = b"\xFE\xFD")]
pub(crate) enum QueryRequest {
    /// Basic stat: https://minecraft.wiki/w/Query#Request_2
    ///
    /// Full stat: https://minecraft.wiki/w/Query#Request_2
    #[deku(id = "0x00")]
    Stat(StatRequest),
    /// See: https://minecraft.wiki/w/Query#Request
    #[deku(id = "0x09")]
    Handshake { session_id: i32 },
}

#[derive(Debug)]
pub(crate) struct StatRequest {
    pub session_id: i32,
    pub challenge_token: i32,
    pub full: bool,
}

impl DekuReader<'_, Endian> for StatRequest {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
        reader: &mut Reader<R>,
        ctx: Endian,
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        let session_id = i32::from_reader_with_ctx(reader, ctx)?;
        let challenge_token = i32::from_reader_with_ctx(reader, ctx)?;
        Ok(StatRequest {
            session_id,
            challenge_token,
            full: !reader.end(), // if there is more data, it's a full stat request
        })
    }
}
