use alloc::vec::Vec;

pub(crate) struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Reader { data }
    }

    pub(crate) fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let (head, rest) = self.data.split_at_checked(len)?;
        self.data = rest;
        Some(head)
    }

    pub(crate) fn u32(&mut self) -> Option<u32> {
        let bytes = self.take(4)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn string(&mut self) -> Option<&'a [u8]> {
        let len = usize::try_from(self.u32()?).ok()?;
        self.take(len)
    }

    pub(crate) fn finished(&self) -> bool {
        self.data.is_empty()
    }
}

pub(crate) fn put_string(out: &mut Vec<u8>, value: &[u8]) {
    let len = u32::try_from(value.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(value);
}
