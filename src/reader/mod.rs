use leb128::read::unsigned as read_varuint;
use std::str::from_utf8;

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

pub enum SectionContent<'a> {
    Type(TypeSection<'a>),
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
    pub fn sections(&'a self) -> SectionsIterator<'a> {
        SectionsIterator(self.payload)
    }
}

macro_rules! try_opt {
    ($ex:expr) => {
        match $ex {
            Some(x) => x,
            None => return None
        }
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
