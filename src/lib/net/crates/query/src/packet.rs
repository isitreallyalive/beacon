use std::{ffi, io};

use deku::{ctx::Endian, prelude::*};

#[derive(Debug, DekuRead)]
#[cfg_attr(test, derive(DekuWrite))]
#[deku(endian = "big", id_type = "u8", magic = b"\xFE\xFD")]
pub enum QueryRequest {
    #[deku(id = "0x00")]
    FullStat,
    #[deku(id = "0x00")]
    BasicStat,
    #[deku(id = "0x09")]
    Handshake,
}

impl QueryRequest {
    /// Maximum size of a c2s packet (full stat)
    // 2 (magic) + 1 (id) + 4 (session id) + 4 (challenge token) + 4 (padding)
    pub const MAX_SIZE: usize = 2 + 1 + 4 + 4 + 4;
}

#[derive(Debug, DekuWrite)]
#[cfg_attr(test, derive(DekuRead))]
#[deku(endian = "big", id_type = "u8")]
pub enum QueryResponse {
    #[deku(id = "0x00")]
    FullStat,
    #[deku(id = "0x00")]
    BasicStat,
    #[deku(id = "0x09")]
    Handshake { token: CString },
}

#[derive(Debug)]
pub struct CString(pub ffi::CString);

impl CString {
    pub fn new(s: &str) -> io::Result<Self> {
        Ok(Self(ffi::CString::new(s)?))
    }
}

impl DekuWriter<Endian> for CString {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        _: Endian,
    ) -> Result<(), DekuError> {
        <ffi::CString as DekuWriter<()>>::to_writer(&self.0, writer, ())
    }
}

#[cfg(test)]
impl DekuReader<'_, Endian> for CString {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
        reader: &mut Reader<R>,
        _: Endian,
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        <ffi::CString as DekuReader<'_, ()>>::from_reader_with_ctx(reader, ()).map(CString)
    }
}
