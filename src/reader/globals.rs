use super::*;
use std::io::Read;

pub struct GlobalSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct GlobalEntryIterator<'a> {
    count: u32,
    iter: &'a [u8]
}

pub struct GlobalEntry {
    pub ty: ValueType,
    pub mutable: bool,
}

impl<'a> GlobalSection<'a> {
    pub fn entries(&self) -> GlobalEntryIterator<'a> {
        GlobalEntryIterator {
            count: self.count,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for GlobalEntryIterator<'a> {
    type Item = Result<GlobalEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let mut ty = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut ty));
        let mutable = try_opt!(read_varuint(&mut self.iter)) != 0;
        Some(Ok(GlobalEntry {
            ty: try_opt!(ValueType::from_int(ty[0]).ok_or(Error::UnknownVariant("value type"))),
            mutable: mutable,
        }))
    }
}
