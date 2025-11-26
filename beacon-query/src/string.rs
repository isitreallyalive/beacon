use std::{ffi, io, sync::Arc};

use deku::{ctx::Endian, prelude::*};

/// A reference-counted C-style string. Makes cloning cheap (necessary for deku).
#[derive(Clone, Default, Hash, PartialEq, Eq)]
pub(crate) struct CString(Arc<ffi::CString>);

impl CString {
    pub fn new<S: Into<Vec<u8>>>(s: S) -> Result<Self, ffi::NulError> {
        Ok(Self(Arc::new(ffi::CString::new(s)?)))
    }
}

impl std::ops::Deref for CString {
    type Target = ffi::CStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DekuWriter<Endian> for CString {
    fn to_writer<W: io::Write + io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        self.0.to_writer(writer, ctx)
    }
}
