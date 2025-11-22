use deku::prelude::*;

/// Maximum size of a serialized QueryRequest in bytes. Happens when FullStat is requested.
/// 
/// MAX_SIZE = magic(2) + id(1) + session(4) + token(4) + padding(4)
pub const MAX_SIZE: usize = 2 + 1 + 4 + 4 + 4;

#[derive(Debug, DekuRead)]
#[deku(endian = "big", id_type = "u8", magic = b"\xFE\xFD")]
pub enum QueryRequest {
    #[deku(id = "0x00")]
    Stat,
    #[deku(id = "0x09")]
    Handshake
}