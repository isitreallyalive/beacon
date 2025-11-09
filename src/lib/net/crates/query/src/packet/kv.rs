use std::io;

use deku::{ctx::Endian, prelude::*};
use indexmap::IndexMap;

use crate::packet::CString;

#[derive(Debug)]
pub struct KeyValue(IndexMap<CString, CString>);

impl std::ops::Deref for KeyValue {
    type Target = IndexMap<CString, CString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> From<[(CString, CString); N]> for KeyValue {
    fn from(kv: [(CString, CString); N]) -> Self {
        let mut map = IndexMap::new();
        for (key, value) in kv {
            map.insert(key.clone(), value.clone());
        }
        KeyValue(map)
    }
}

impl DekuWriter<Endian> for KeyValue {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        for (key, value) in &self.0 {
            key.to_writer(writer, ctx)?;
            value.to_writer(writer, ctx)?;
        }
        0u8.to_writer(writer, ctx)?; // null terminator
        Ok(())
    }
}

#[cfg(test)]
impl DekuReader<'_, Endian> for KeyValue {
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

        Ok(KeyValue(map))
    }
}
