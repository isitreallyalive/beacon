use std::io;

use crate::{write_null, write_string};

// padding bytes for full stat
pub(crate) const HEADER_PADDING: [u8; 11] = [
    0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
];
pub(crate) const KV_PADDING: [u8; 10] =
    [0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00];

// hardcoded stats
pub(crate) const GAMETYPE: &str = "SMP";
pub(crate) const GAME_ID: &str = "MINECRAFT";

pub(crate) struct StatsResponse<'a> {
    pub motd: &'a str,
    pub map: &'a str,
    pub numplayers: &'a str,
    pub maxplayers: &'a str,
    pub hostport: &'a str,
    pub hostip: &'a str,
}

impl StatsResponse<'_> {
    /// Respond in basic stat format
    pub fn basic<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for value in [
            self.motd,
            GAMETYPE,
            self.map,
            self.numplayers,
            self.maxplayers,
            self.hostport,
            self.hostip,
        ] {
            write_string(writer, value)?;
        }
        Ok(())
    }

    /// Respond in full stat format
    pub fn full<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        // note: vanilla server would cache this every 5 seconds, but we won't
        writer.write_all(&HEADER_PADDING)?;
        for (key, value) in [
            ("hostname", self.motd),
            ("gametype", GAMETYPE),
            ("game_id", GAME_ID),
            ("version", beacon_config::VERSION),
            ("plugins", ""),
            ("map", self.map),
            ("numplayers", self.numplayers),
            ("maxplayers", self.maxplayers),
            ("hostport", self.hostport),
            ("hostip", self.hostip),
        ] {
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
