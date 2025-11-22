use std::ffi::CString;

use deku::prelude::*;

#[derive(Debug, DekuWrite)]
#[deku(endian = "big", id_type = "u8")]
pub enum QueryResponse {
    /// Basic stat: https://minecraft.wiki/w/Query#Response_2
    ///
    /// Full stat: https://minecraft.wiki/w/Query#Response_3
    #[deku(id = "0x00")]
    Stat,
    /// See: https://minecraft.wiki/w/Query#Response
    #[deku(id = "0x09")]
    Handshake {
        session_id: i32,
        challenge_token: CString,
    },
}
