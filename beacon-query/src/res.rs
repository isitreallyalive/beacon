use std::ffi::CString;

use deku::prelude::*;

use crate::kv::KeyValue;

pub const KV_MARKER: [u8; 11] = [
    0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
]; // "splitnum\0\x80\0"
pub const PLAYER_MARKER: [u8; 10] = [0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00]; // "\x01player_\0\0"

#[derive(DekuWrite)]
#[deku(endian = "big", id_type = "u8")]
pub enum QueryResponse {
    /// See: https://minecraft.wiki/w/Query#Response_2
    #[deku(id = "0x00")]
    BasicStat {
        session_id: i32,
        motd: CString,
        game_type: CString,
        map: CString,
        num_players: CString,
        max_players: CString,
        #[deku(endian = "little")]
        host_port: u16,
        host_ip: CString,
    },
    /// See: https://minecraft.wiki/w/Query#Response_3
    #[deku(id = "0x00")]
    FullStat {
        session_id: i32,
        kv_marker: [u8; 11],
        kv: KeyValue,
        player_marker: [u8; 10],
        players: Vec<CString>,
        nul: u8
    },
    /// See: https://minecraft.wiki/w/Query#Response
    #[deku(id = "0x09")]
    Handshake {
        session_id: i32,
        challenge_token: CString,
    },
}
