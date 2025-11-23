use std::{ffi::CString, io, ops};

use deku::{ctx::Endian, prelude::*};
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct KeyValue(IndexMap<CString, CString>);

impl ops::Deref for KeyValue {
    type Target = IndexMap<CString, CString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for KeyValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
        // write terminating null
        0u8.to_writer(writer, ctx)
    }
}
