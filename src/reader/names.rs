use super::*;
use std::str::from_utf8;

pub struct NameSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct NameEntryIterator<'a> {
    count: u32,
    local_count: u32,
    iter: &'a [u8]
}

pub enum NameEntry<'a> {
    Function(&'a str),
    Local(&'a str),
}

impl<'a> NameSection<'a> {
    pub fn entries(&self) -> NameEntryIterator<'a> {
        NameEntryIterator {
            count: self.count,
            local_count: 0,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for NameEntryIterator<'a> {
    type Item = Result<NameEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 && self.local_count == 0 {
            return None
        }
        if self.local_count > 0 {
            self.local_count -= 1;
            let len = try_opt!(read_varuint(&mut self.iter)) as usize;
            let name = &self.iter[..len];
            self.iter = &self.iter[len..];
            let name = try_opt!(from_utf8(name));
            return Some(Ok(NameEntry::Local(name)))
        }
        self.count -= 1;
        let len = try_opt!(read_varuint(&mut self.iter)) as usize;
        let name = &self.iter[..len];
        self.iter = &self.iter[len..];
        let name = try_opt!(from_utf8(name));
        let count = try_opt!(read_varuint(&mut self.iter)) as u32;
        self.local_count = count;
        Some(Ok(NameEntry::Function(name)))
    }
}
