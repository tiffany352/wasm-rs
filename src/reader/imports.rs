use super::*;
use std::str::from_utf8;
use std::io::Read;

pub struct ImportSection<'a>(pub &'a [u8], pub usize);

pub struct ImportEntryIterator<'a>(&'a [u8], usize);

pub struct ImportEntry<'a> {
    pub module: &'a str,
    pub field: &'a str,
    pub contents: ImportEntryContents,
}

pub enum ImportEntryContents {
    Function(u32),
    Table {
        element_type: u8,
        limits: ResizableLimits
    },
    Memory(ResizableLimits),
    Global {
        ty: ValueType,
        mutable: bool
    },
}

pub struct ResizableLimits {
    pub initial: u32,
    pub maximum: Option<u32>,
}

impl<'a> ImportSection<'a> {
    pub fn entries(&self) -> ImportEntryIterator<'a> {
        ImportEntryIterator(self.0, self.1)
    }
}

impl<'a> Iterator for ImportEntryIterator<'a> {
    type Item = Result<ImportEntry<'a>, Error>;

    fn next(&mut self) -> Option<Result<ImportEntry<'a>, Error>> {
        if self.1 == 0 {
            return None
        }
        self.1 -= 1;
        let mlen = try_opt!(read_varuint(&mut self.0));
        let module = {
            let res = &self.0[..mlen as usize];
            self.0 = &self.0[mlen as usize..];
            try_opt!(from_utf8(res))
        };
        let flen = try_opt!(read_varuint(&mut self.0));
        let field = {
            let res = &self.0[..flen as usize];
            self.0 = &self.0[flen as usize..];
            try_opt!(from_utf8(res))
        };
        let mut kind = [0; 1];
        try_opt!((&mut self.0).read_exact(&mut kind));
        let kind = try_opt!(ExternalKind::from_int(kind[0]).ok_or(Error::UnknownVariant("external kind")));
        let contents = match kind {
            ExternalKind::Function => ImportEntryContents::Function(try_opt!(
                read_varuint(&mut self.0)) as u32
            ),
            ExternalKind::Table => ImportEntryContents::Table {
                element_type: {
                    let mut ty = [0; 1];
                    try_opt!((&mut self.0).read_exact(&mut ty));
                    ty[0]
                },
                limits: try_opt!(ResizableLimits::parse(&mut self.0)),
            },
            ExternalKind::Memory => ImportEntryContents::Memory(
                try_opt!(ResizableLimits::parse(&mut self.0))
            ),
            ExternalKind::Global => ImportEntryContents::Global {
                ty: {
                    let mut ty = [0; 1];
                    try_opt!((&mut self.0).read_exact(&mut ty));
                    try_opt!(ValueType::from_int(ty[0]).ok_or(Error::UnknownVariant("value type")))
                },
                mutable: try_opt!(read_varuint(&mut self.0)) != 0,
            },
        };
        Some(Ok(ImportEntry {
            module: module,
            field: field,
            contents: contents,
        }))
    }
}

impl ResizableLimits {
    fn parse(mut iter: &mut &[u8]) -> Result<ResizableLimits, Error> {
        let flags = try!(read_varuint(iter));
        let initial = try!(read_varuint(iter));
        let maximum = if flags & 0x1 != 0 {
            Some(try!(read_varuint(iter)))
        } else {
            None
        };
        Ok(ResizableLimits {
            initial: initial as u32,
            maximum: maximum.map(|x| x as u32),
        })
    }
}
