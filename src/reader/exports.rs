use super::*;
use std::io::Read;
use std::str::from_utf8;

pub struct ExportSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct ExportEntryIterator<'a> {
    count: u32,
    iter: &'a [u8]
}

pub struct ExportEntry<'a> {
    pub field: &'a str,
    pub kind: ExternalKind,
    pub index: u32,
}

impl<'a> ExportSection<'a> {
    pub fn entries(&self) -> ExportEntryIterator<'a> {
        ExportEntryIterator {
            count: self.count,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for ExportEntryIterator<'a> {
    type Item = Result<ExportEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let flen = try_opt!(read_varuint(&mut self.iter)) as usize;
        let field = {
            let res = &self.iter[..flen];
            self.iter = &self.iter[flen..];
            try_opt!(from_utf8(res))
        };
        let mut kind = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut kind));
        let kind = try_opt!(ExternalKind::from_int(kind[0]).ok_or(Error::UnknownVariant("external kind")));
        let index = try_opt!(read_varuint(&mut self.iter)) as u32;
        Some(Ok(ExportEntry {
            field: field,
            kind: kind,
            index: index,
        }))
    }
}
