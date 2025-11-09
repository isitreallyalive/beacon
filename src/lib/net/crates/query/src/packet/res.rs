use std::io;

use deku::{ctx::Endian, prelude::*};

use crate::{
    packet::{CString, Map},
    stats::Stats,
};

const KV_MARKER: [u8; 11] = [
    0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
]; // "splitnum\0\x80\0"
const PLAYER_MARKER: [u8; 10] = [0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00]; // "\x01player_\0\0"

#[derive(Debug, DekuWrite)]
#[cfg_attr(test, derive(DekuRead))]
#[deku(endian = "big", id_type = "u8")]
pub enum QueryResponse {
    #[deku(id = "0x00")]
    Stat(StatResponseKind),
    #[deku(id = "0x09")]
    Handshake {
        session_id: i32,
        challenge_token: CString,
    },
}

#[derive(Debug)]
pub enum StatResponseKind {
    Full {
        session_id: i32,
        kv: Map,
        players: Vec<CString>,
    },
    Basic {
        session_id: i32,
        basic: Stats,
    },
}

impl DekuWriter<Endian> for StatResponseKind {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        match self {
            StatResponseKind::Full {
                session_id,
                kv,
                players,
            } => {
                session_id.to_writer(writer, ctx)?;
                KV_MARKER.to_writer(writer, ctx)?;
                kv.to_writer(writer, ctx)?;
                0u8.to_writer(writer, ctx)?; // null terminator
                PLAYER_MARKER.to_writer(writer, ctx)?;
                for player in players {
                    player.to_writer(writer, ctx)?;
                }
                0u8.to_writer(writer, ctx)?; // null terminator
            }
            StatResponseKind::Basic { session_id, basic } => {
                session_id.to_writer(writer, ())?;
                basic.to_writer(writer, ())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
impl DekuReader<'_, Endian> for StatResponseKind {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
        reader: &mut Reader<R>,
        ctx: Endian,
    ) -> Result<Self, DekuError> {
        use io::Seek;

        let session_id = i32::from_reader_with_ctx(reader, ())?;

        // try to read the kv marker
        let pos = reader
            .stream_position()
            .map_err(|e| DekuError::Io(e.kind()))?;
        let mut kv_marker = [0u8; 11];
        reader.read_bytes(11, &mut kv_marker, deku::ctx::Order::Lsb0)?;

        if kv_marker == KV_MARKER {
            // full stat response
            let kv = Map::from_reader_with_ctx(reader, ctx)?;

            // read player marker and list
            let mut player_marker = [0u8; 10];
            reader.read_bytes(10, &mut player_marker, deku::ctx::Order::Lsb0)?;

            let mut players = Vec::new();
            if player_marker == PLAYER_MARKER {
                loop {
                    let mut peek = [0u8; 1];
                    reader.read_bytes(1, &mut peek, deku::ctx::Order::Lsb0)?;
                    if peek[0] == 0 {
                        break;
                    }
                    reader
                        .seek(io::SeekFrom::Current(-1))
                        .map_err(|e| DekuError::Io(e.kind()))?;
                    players.push(CString::from_reader_with_ctx(reader, ctx)?);
                }
            }

            Ok(StatResponseKind::Full {
                session_id,
                kv,
                players,
            })
        } else {
            // basic stat response - rewind and read as Stats
            reader
                .seek(io::SeekFrom::Start(pos))
                .map_err(|e| DekuError::Io(e.kind()))?;
            Ok(StatResponseKind::Basic {
                session_id,
                basic: Stats::from_reader_with_ctx(reader, ())?,
            })
        }
    }
}
