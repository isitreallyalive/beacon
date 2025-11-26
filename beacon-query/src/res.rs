use std::sync::LazyLock;

use deku::prelude::*;

use crate::{kv::KeyValue, string::CString};

#[derive(DekuWrite)]
#[deku(endian = "big", id_type = "u8")]
pub(crate) enum QueryResponse {
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
        #[deku(pad_bytes_before = "11")]
        kv: KeyValue,
        #[deku(pad_bytes_before = "10", pad_bytes_after = "1")]
        players: Vec<CString>,
    },
    /// See: https://minecraft.wiki/w/Query#Response
    #[deku(id = "0x09")]
    Handshake {
        session_id: i32,
        challenge_token: CString,
    },
}

macro_rules! lazy_string {
    ($(
        $name:ident = $value:expr $(;)? // optional trailing semicolon
    );+) => {
        $(
            pub static $name: LazyLock<CString> = LazyLock::new(|| CString::new($value).unwrap());
        )+
    };
}

lazy_string! {
    // full stat keys
    HOSTNAME_KEY = "hostname";
    GAMETYPE_KEY = "gametype";
    GAME_ID_KEY = "game_id";
    VERSION_KEY = "version";
    PLUGINS_KEY = "plugins";
    MAP_KEY = "map";
    NUMPLAYERS_KEY = "numplayers";
    MAXPLAYERS_KEY = "maxplayers";
    HOSTPORT_KEY = "hostport";
    HOSTIP_KEY = "hostip";

    // hard-coded full stat values
    GAME_TYPE = "SMP";
    GAME_ID = "MINECRAFT";
    VERSION = beacon_data::LATEST_SUPPORTED;
    PLUGINS = ""; // no plugins
}
