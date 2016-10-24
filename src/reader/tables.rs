use super::*;
use std::io::Read;

pub struct TableSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct TableEntryIterator<'a> {
    count: u32,
    iter: &'a [u8]
}

pub struct TableEntry {
    pub ty: u8,
    pub limits: ResizableLimits,
}

impl<'a> TableSection<'a> {
    pub fn entries(&self) -> TableEntryIterator<'a> {
        TableEntryIterator {
            count: self.count,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for TableEntryIterator<'a> {
    type Item = Result<TableEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let mut ty = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut ty));
        let limits = try_opt!(ResizableLimits::parse(&mut self.iter));
        Some(Ok(TableEntry {
            ty: ty[0],
            limits: limits,
        }))
    }
}
