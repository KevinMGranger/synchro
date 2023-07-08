use super::prelude::*;

pub(crate) struct OwnedCWSTR {
    buf: Vec<u16>,
}

impl OwnedCWSTR {
    /// this shouldn't be safe tbh but I'm just trying to deal with allocation
    fn get_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.buf.as_ptr())
    }
}
