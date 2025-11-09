use std::io;

use deku::{ctx::Endian, prelude::*};
use indexmap::IndexMap;

use crate::packet::CString;

/// An ordered, key-value map.
#[derive(Debug)]
pub struct Map(IndexMap<CString, CString>);

impl std::ops::Deref for Map {
    type Target = IndexMap<CString, CString>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> From<[(CString, CString); N]> for Map {
    fn from(kv: [(CString, CString); N]) -> Self {
        Map(kv.into_iter().collect())
    }
}

impl DekuWriter<Endian> for Map {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        for (key, value) in &self.0 {
            key.to_writer(writer, ctx)?;
            value.to_writer(writer, ctx)?;
        }
        0u8.to_writer(writer, ctx)?;
        Ok(())
    }
}

#[cfg(test)]
impl DekuReader<'_, Endian> for Map {
    fn from_reader_with_ctx<R: io::Read + io::Seek>(
        reader: &mut Reader<R>,
        ctx: Endian,
    ) -> Result<Self, DekuError> {
        let mut map = IndexMap::new();
        loop {
            let key = CString::from_reader_with_ctx(reader, ctx)?;
            if key.0.is_empty() {
                break;
            }
            let value = CString::from_reader_with_ctx(reader, ctx)?;
            map.insert(key, value);
        }
        Ok(Map(map))
    }
}
