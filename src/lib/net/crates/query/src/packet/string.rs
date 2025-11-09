use std::{ffi, io};

use deku::{ctx::Endian, prelude::*};

/// A C-style null-terminated string.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CString(pub ffi::CString);

impl CString {
    pub fn new(s: &str) -> io::Result<Self> {
        Ok(Self(ffi::CString::new(s)?))
    }
}

impl std::ops::Deref for CString {
    type Target = ffi::CString;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DekuWriter<Endian> for CString {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        _: Endian,
    ) -> Result<(), DekuError> {
        <ffi::CString as DekuWriter<()>>::to_writer(self, writer, ())
    }
}

#[cfg(test)]
impl DekuReader<'_, Endian> for CString {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
        reader: &mut Reader<R>,
        _: Endian,
    ) -> Result<Self, DekuError> {
        <ffi::CString as DekuReader<'_, ()>>::from_reader_with_ctx(reader, ()).map(CString)
    }
}
