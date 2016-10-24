use leb128::read::unsigned as read_varuint;
use std::str::from_utf8;
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::Read;

macro_rules! try_opt {
    ($ex:expr) => {
        match $ex {
            Some(x) => x,
            None => return None
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

pub struct TypeSection<'a>(&'a [u8]);

pub struct TypeEntryIterator<'a>(&'a [u8]);

pub enum TypeEntry<'a> {
    Function(FunctionType<'a>),
}

pub struct ParamsIterator<'a>(&'a [u8]);

pub struct FunctionType<'a> {
    params_raw: &'a [u8],
    pub return_type: Option<ValueType>,
}

pub struct ImportSection<'a>(&'a [u8]);

pub struct ImportEntryIterator<'a>(&'a [u8]);

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

pub struct FunctionSection<'a>(&'a [u8]);

pub struct FunctionEntryIterator<'a>(&'a [u8]);

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
    pub fn new(mut stream: &'a [u8]) -> Option<Module<'a>> {
        let mut magic = [0; 4];
        try_opt!((&mut stream).read_exact(&mut magic).ok());
        if &magic != b"\0asm" {
            return None
        }
        let version = try_opt!((&mut stream).read_u32::<LittleEndian>().ok());
        Some(Module {
            version: version,
            payload: stream
        })
    }

    pub fn sections(&'a self) -> SectionsIterator<'a> {
        SectionsIterator(self.payload)
    }
}

impl<'a> Iterator for SectionsIterator<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Section<'a>> {
        let id = try_opt!(read_varuint(&mut self.0).ok());
        let id = try_opt!(SectionType::from_int(id as u8));
        let plen = try_opt!(read_varuint(&mut self.0).ok());
        let start = self.0.len() as u64;
        let nlen = if id == SectionType::Named {
            try_opt!(read_varuint(&mut self.0).ok())
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
        Some(Section {
            id: id,
            name: try_opt!(from_utf8(name).ok()),
            payload: payload,
        })
    }
}

impl<'a> Section<'a> {
    pub fn content(&self) -> Option<SectionContent<'a>> {
        match self.id {
            SectionType::Named => None,
            SectionType::Type => {
                Some(SectionContent::Type(TypeSection(self.payload)))
            },
            SectionType::Import => {
                Some(SectionContent::Import(ImportSection(self.payload)))
            },
            SectionType::Function => {
                Some(SectionContent::Function(FunctionSection(self.payload)))
            },
            SectionType::Start => {
                let mut r = self.payload;
                let index = try_opt!(read_varuint(&mut r).ok());
                Some(SectionContent::Start(index as u32))
            },
            _ => None
        }
    }
}

impl<'a> TypeSection<'a> {
    pub fn entries(&self) -> TypeEntryIterator<'a> {
        TypeEntryIterator(self.0)
    }
}

impl<'a> Iterator for TypeEntryIterator<'a> {
    type Item = TypeEntry<'a>;

    fn next(&mut self) -> Option<TypeEntry<'a>> {
        let form = try_opt!(read_varuint(&mut self.0).ok());
        if form != 0x40 {
            return None
        }
        let param_count = try_opt!(read_varuint(&mut self.0).ok());
        let params = if param_count > self.0.len() as u64 {
            return None
        } else {
            let res = &self.0[..param_count as usize];
            self.0 = &self.0[param_count as usize..];
            res
        };
        let return_count = try_opt!(read_varuint(&mut self.0).ok());
        let return_ty = if return_count > 0 {
            if self.0.len() < 1 {
                return None
            }
            let res = self.0[0];
            self.0 = &self.0[1..];
            Some(try_opt!(ValueType::from_int(res)))
        } else {
            None
        };
        Some(TypeEntry::Function(FunctionType {
            params_raw: params,
            return_type: return_ty
        }))
    }
}

impl<'a> FunctionType<'a> {
    pub fn params(&self) -> ParamsIterator<'a> {
        ParamsIterator(self.params_raw)
    }
}

impl<'a> Iterator for ParamsIterator<'a> {
    type Item = ValueType;

    fn next(&mut self) -> Option<ValueType> {
        if self.0.len() < 1 {
            return None
        }
        let res = self.0[0];
        self.0 = &self.0[1..];
        Some(try_opt!(ValueType::from_int(res)))
    }
}

impl<'a> ImportSection<'a> {
    pub fn entries(&self) -> ImportEntryIterator<'a> {
        ImportEntryIterator(self.0)
    }
}

impl<'a> Iterator for ImportEntryIterator<'a> {
    type Item = ImportEntry<'a>;

    fn next(&mut self) -> Option<ImportEntry<'a>> {
        let mlen = try_opt!(read_varuint(&mut self.0).ok());
        let module = {
            let res = &self.0[..mlen as usize];
            self.0 = &self.0[mlen as usize..];
            try_opt!(from_utf8(res).ok())
        };
        let flen = try_opt!(read_varuint(&mut self.0).ok());
        let field = {
            let res = &self.0[..flen as usize];
            self.0 = &self.0[flen as usize..];
            try_opt!(from_utf8(res).ok())
        };
        let mut kind = [0; 1];
        try_opt!((&mut self.0).read_exact(&mut kind).ok());
        let kind = try_opt!(ExternalKind::from_int(kind[0]));
        let contents = match kind {
            ExternalKind::Function => ImportEntryContents::Function(try_opt!(
                read_varuint(&mut self.0).ok()) as u32
            ),
            ExternalKind::Table => ImportEntryContents::Table {
                element_type: {
                    let mut ty = [0; 1];
                    try_opt!((&mut self.0).read_exact(&mut ty).ok());
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
                    try_opt!((&mut self.0).read_exact(&mut ty).ok());
                    try_opt!(ValueType::from_int(ty[0]))
                },
                mutable: try_opt!(read_varuint(&mut self.0).ok()) != 0,
            },
        };
        Some(ImportEntry {
            module: module,
            field: field,
            contents: contents,
        })
    }
}

impl ResizableLimits {
    fn parse(mut iter: &mut &[u8]) -> Option<ResizableLimits> {
        let flags = try_opt!(read_varuint(iter).ok());
        let initial = try_opt!(read_varuint(iter).ok());
        let maximum = if flags & 0x1 != 0 {
            Some(try_opt!(read_varuint(iter).ok()))
        } else {
            None
        };
        Some(ResizableLimits {
            initial: initial as u32,
            maximum: maximum.map(|x| x as u32),
        })
    }
}

impl<'a> FunctionSection<'a> {
    pub fn types(&self) -> FunctionEntryIterator<'a> {
        FunctionEntryIterator(self.0)
    }
}

impl<'a> Iterator for FunctionEntryIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        read_varuint(&mut self.0).ok().map(|x| x as u32)
    }
}
