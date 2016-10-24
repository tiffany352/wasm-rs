use leb128::read::unsigned as read_varuint;
use std::str::from_utf8;
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::Read;
use std::io;

macro_rules! try_opt {
    ($ex:expr) => {
        match $ex {
            Ok(x) => x,
            Err(e) => return Some(Err(e.into()))
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NotWasm(err: io::Error) {
            cause(err)
            description("Not a valid WebAssembly binary")
            display("Not a valid WebAssembly binary: {}", err)
        }
        Io(err: io::Error) {
            description("IO error")
            cause(err)
            display("{}", err)
            from()
        }
        Leb128(err: ::leb128::read::Error) {
            description("Malformed LEB128 integer")
            cause(err)
            display("Malformed LEB128 integer: {}", err)
            from()
        }
        UnknownVariant(of: &'static str) {
            description("Unknown enum variant")
            display("Unknown enum variant for {}", of)
        }
        Utf8(err: ::std::str::Utf8Error) {
            description("UTF-8 error")
            cause(err)
            display("UTF-8 error: {}", err)
            from()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SectionType {
    Named = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11
}

impl SectionType {
    fn from_int(v: u8) -> Option<SectionType> {
        Some(match v {
            0 => SectionType::Named,
            1 => SectionType::Type,
            2 => SectionType::Import,
            3 => SectionType::Function,
            4 => SectionType::Table,
            5 => SectionType::Memory,
            6 => SectionType::Global,
            7 => SectionType::Export,
            8 => SectionType::Start,
            9 => SectionType::Element,
            10 => SectionType::Code,
            11 => SectionType::Data,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValueType {
    I32 = 1,
    I64 = 2,
    F32 = 3,
    F64 = 4,
}

impl ValueType {
    fn from_int(v: u8) -> Option<ValueType> {
        Some(match v {
            1 => ValueType::I32,
            2 => ValueType::I64,
            3 => ValueType::F32,
            4 => ValueType::F64,
            _ => return None
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ExternalKind {
    Function = 0,
    Table = 1,
    Memory = 2,
    Global = 3,
}

impl ExternalKind {
    fn from_int(v: u8) -> Option<ExternalKind> {
        Some(match v {
            0 => ExternalKind::Function,
            1 => ExternalKind::Table,
            2 => ExternalKind::Memory,
            3 => ExternalKind::Global,
            _ => return None,
        })
    }
}

pub struct TypeSection<'a>(&'a [u8], usize);

pub struct TypeEntryIterator<'a>(&'a [u8], usize);

pub enum TypeEntry<'a> {
    Function(FunctionType<'a>),
}

pub struct ParamsIterator<'a>(&'a [u8], usize);

pub struct FunctionType<'a> {
    params_count: usize,
    params_raw: &'a [u8],
    pub return_type: Option<ValueType>,
}

pub struct ImportSection<'a>(&'a [u8], usize);

pub struct ImportEntryIterator<'a>(&'a [u8], usize);

pub struct ResizableLimits {
    pub initial: u32,
    pub maximum: Option<u32>,
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

pub struct ImportEntry<'a> {
    pub module: &'a str,
    pub field: &'a str,
    pub contents: ImportEntryContents,
}

pub struct FunctionSection<'a>(&'a [u8], usize);

pub struct FunctionEntryIterator<'a>(&'a [u8], usize);

pub enum SectionContent<'a> {
    Type(TypeSection<'a>),
    Import(ImportSection<'a>),
    Function(FunctionSection<'a>),
    Start(u32),
}

pub struct Section<'a> {
    pub id: SectionType,
    pub name: &'a str,
    pub payload: &'a [u8],
}

pub struct Module<'a> {
    pub version: u32,
    pub payload: &'a [u8],
}

pub struct SectionsIterator<'a>(&'a [u8]);

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

impl<'a> TypeSection<'a> {
    pub fn entries(&self) -> TypeEntryIterator<'a> {
        TypeEntryIterator(self.0, self.1)
    }
}

impl<'a> Iterator for TypeEntryIterator<'a> {
    type Item = Result<TypeEntry<'a>, Error>;

    fn next(&mut self) -> Option<Result<TypeEntry<'a>, Error>> {
        if self.1 == 0 {
            return None
        }
        self.1 -= 1;
        let form = try_opt!(read_varuint(&mut self.0));
        if form != 0x40 {
            return Some(Err(Error::UnknownVariant("type entry form")))
        }
        let param_count = try_opt!(read_varuint(&mut self.0));
        let params = if param_count > self.0.len() as u64 {
            return Some(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "param_count is larger than remaining space"
            ))))
        } else {
            let res = &self.0[..param_count as usize];
            self.0 = &self.0[param_count as usize..];
            res
        };
        let return_count = try_opt!(read_varuint(&mut self.0));
        let return_ty = if return_count > 0 {
            if self.0.len() < 1 {
                return Some(Err(Error::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "return_count is larger than remaining space"
                ))))
            }
            let res = self.0[0];
            self.0 = &self.0[1..];
            Some(try_opt!(ValueType::from_int(res).ok_or(Error::UnknownVariant("value type"))))
        } else {
            None
        };
        Some(Ok(TypeEntry::Function(FunctionType {
            params_count: param_count as usize,
            params_raw: params,
            return_type: return_ty
        })))
    }
}

impl<'a> FunctionType<'a> {
    pub fn params(&self) -> ParamsIterator<'a> {
        ParamsIterator(self.params_raw, self.params_count)
    }
}

impl<'a> Iterator for ParamsIterator<'a> {
    type Item = Result<ValueType, Error>;

    fn next(&mut self) -> Option<Result<ValueType, Error>> {
        if self.1 == 0 {
            return None
        }
        self.1 -= 1;
        if self.0.len() < 1 {
            return Some(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof, "number of params is larger than available space"
            ))))
        }
        let res = self.0[0];
        self.0 = &self.0[1..];
        Some(Ok(try_opt!(ValueType::from_int(res).ok_or(Error::UnknownVariant("value type")))))
    }
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

impl<'a> FunctionSection<'a> {
    pub fn types(&self) -> FunctionEntryIterator<'a> {
        FunctionEntryIterator(self.0, self.1)
    }
}

impl<'a> Iterator for FunctionEntryIterator<'a> {
    type Item = Result<u32, Error>;

    fn next(&mut self) -> Option<Result<u32, Error>> {
        if self.1 == 0 {
            return None
        }
        self.1 -= 1;
        Some(read_varuint(&mut self.0).map(|x| x as u32).map_err(|x| x.into()))
    }
}
