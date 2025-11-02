use std::io;

use crate::{write_null, write_string};

// padding bytes for full stat
const HEADER_PADDING: [u8; 11] = [
    0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
];
const KV_PADDING: [u8; 10] = [0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00];

// hardcoded stats
const GAMETYPE: &str = "SMP";
const GAME_ID: &str = "MINECRAFT";

// keys for full stat
const STAT_KEYS: [&str; 10] = [
    "hostname",
    "gametype",
    "game_id",
    "version",
    "plugins",
    "map",
    "numplayers",
    "maxplayers",
    "hostport",
    "hostip",
];

pub(crate) struct StatsResponse<'a> {
    pub motd: &'a str,
    pub map: &'a str,
    pub numplayers: &'a str,
    pub maxplayers: &'a str,
    pub hostport: &'a str,
    pub hostip: &'a str,
}

impl StatsResponse<'_> {
    const fn values(&self) -> [&str; 10] {
        [
            self.motd,
            GAMETYPE,
            GAME_ID,
            beacon_config::VERSION,
            "",
            self.map,
            self.numplayers,
            self.maxplayers,
            self.hostport,
            self.hostip,
        ]
    }

    /// Respond in basic stat format
    pub fn basic<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for value in self.values() {
            write_string(writer, value)?;
        }
        Ok(())
    }

    /// Respond in full stat format
    pub fn full<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        // note: vanilla server would cache this every 5 seconds, but we won't
        writer.write_all(&HEADER_PADDING)?;
        for (key, value) in STAT_KEYS.iter().zip(self.values()) {
            write_string(writer, key)?;
            write_string(writer, value)?;
        }
        write_null(writer)?;
        writer.write_all(&KV_PADDING)?;
        // todo: player usernames
        write_null(writer)?;
        Ok(())
    }
}
