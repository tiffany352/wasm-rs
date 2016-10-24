use super::*;
use std::io;
use byteorder::{LittleEndian, ReadBytesExt};
use std::str::from_utf8;
use std::io::Read;

pub struct Module<'a> {
    pub version: u32,
    pub payload: &'a [u8],
}

pub struct SectionsIterator<'a>(&'a [u8]);

pub struct Section<'a> {
    pub id: SectionType,
    pub name: &'a str,
    pub payload: &'a [u8],
}

pub enum SectionContent<'a> {
    Type(TypeSection<'a>),
    Import(ImportSection<'a>),
    Function(FunctionSection<'a>),
    Start(u32),
}

impl<'a> Module<'a> {
    pub fn new(mut stream: &'a [u8]) -> Result<Module<'a>, Error> {
        let mut magic = [0; 4];
        try!((&mut stream).read_exact(&mut magic).map_err(Error::NotWasm));
        if &magic != b"\0asm" {
            return Err(Error::NotWasm(io::Error::new(
                io::ErrorKind::InvalidData, "Magic number did not match"
            )))
        }
        let version = try!((&mut stream).read_u32::<LittleEndian>());
        Ok(Module {
            version: version,
            payload: stream
        })
    }

    pub fn sections(&'a self) -> SectionsIterator<'a> {
        SectionsIterator(self.payload)
    }
}

impl<'a> Iterator for SectionsIterator<'a> {
    type Item = Result<Section<'a>, Error>;

    fn next(&mut self) -> Option<Result<Section<'a>, Error>> {
        let id = try_opt!(read_varuint(&mut self.0));
        let id = try_opt!(SectionType::from_int(id as u8).ok_or(Error::UnknownVariant("section type")));
        let plen = try_opt!(read_varuint(&mut self.0));
        let start = self.0.len() as u64;
        let nlen = if id == SectionType::Named {
            try_opt!(read_varuint(&mut self.0))
        } else {
            0
        };
        let name = if nlen > self.0.len() as u64 {
            return None
        } else {
            let res = &self.0[0..nlen as usize];
            self.0 = &self.0[nlen as usize..];
            res
        };
        let plen = plen - (start - self.0.len() as u64);
        let payload = if plen > self.0.len() as u64 {
            return None
        } else {
            let res = &self.0[0..plen as usize];
            self.0 = &self.0[plen as usize..];
            res
        };
        Some(Ok(Section {
            id: id,
            name: try_opt!(from_utf8(name)),
            payload: payload,
        }))
    }
}

impl<'a> Section<'a> {
    pub fn content(&self) -> Result<SectionContent<'a>, Error> {
        match self.id {
            SectionType::Type => {
                let mut iter = self.payload;
                let count = try!(read_varuint(&mut iter)) as usize;
                Ok(SectionContent::Type(TypeSection(iter, count)))
            },
            SectionType::Import => {
                let mut iter = self.payload;
                let count = try!(read_varuint(&mut iter)) as usize;
                Ok(SectionContent::Import(ImportSection(self.payload, count)))
            },
            SectionType::Function => {
                let mut iter = self.payload;
                let count = try!(read_varuint(&mut iter)) as usize;
                Ok(SectionContent::Function(FunctionSection(self.payload, count)))
            },
            SectionType::Start => {
                let mut r = self.payload;
                let index = try!(read_varuint(&mut r));
                Ok(SectionContent::Start(index as u32))
            },
            _ => Err(Error::UnknownVariant("section type"))
        }
    }
}
