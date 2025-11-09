use std::{cell::LazyCell, io};

use beacon_config::VERSION;
use deku::prelude::*;

use crate::packet::{CString, StatResponseKind};

pub const GAMETYPE: LazyCell<CString> = LazyCell::new(|| CString::new("SMP").unwrap());
pub const GAME_ID: LazyCell<CString> = LazyCell::new(|| CString::new("MINECRAFT").unwrap());

#[derive(Debug, DekuWrite)]
#[cfg_attr(test, derive(DekuRead))]
#[deku(endian = "big")]
pub struct Stats {
    pub motd: CString,
    pub gametype: CString,
    pub map: CString,
    pub num_players: CString,
    pub max_players: CString,
    #[deku(endian = "little")]
    pub host_port: u16,
    pub host_ip: CString,
}

impl Stats {
    pub const fn basic(self, session_id: i32) -> io::Result<StatResponseKind> {
        Ok(StatResponseKind::Basic {
            session_id,
            basic: self,
        })
    }

    pub fn full(self, session_id: i32) -> io::Result<StatResponseKind> {
        Ok(StatResponseKind::Full {
            session_id,
            kv: [
                (CString::new("hostname")?, self.motd),
                (CString::new("gametype")?, self.gametype),
                (CString::new("game_id")?, GAME_ID.clone()),
                (CString::new("version")?, CString::new(VERSION)?),
                (CString::new("plugins")?, CString::new("")?),
                (CString::new("map")?, self.map),
                (CString::new("numplayers")?, self.num_players),
                (CString::new("maxplayers")?, self.max_players),
                (
                    CString::new("hostport")?,
                    CString::new(&format!("{}", self.host_port))?,
                ),
                (CString::new("hostip")?, self.host_ip),
            ]
            .into(),
            players: vec![],
        })
    }
}
