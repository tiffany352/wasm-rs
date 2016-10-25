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
    pub fn from_int(v: u8) -> Option<SectionType> {
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
    pub fn from_int(v: u8) -> Option<ValueType> {
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
    pub fn from_int(v: u8) -> Option<ExternalKind> {
        Some(match v {
            0 => ExternalKind::Function,
            1 => ExternalKind::Table,
            2 => ExternalKind::Memory,
            3 => ExternalKind::Global,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InlineSignatureType {
    Empty = 0,
    I32 = 1,
    I64 = 2,
    F32 = 3,
    F64 = 4,
}

impl InlineSignatureType {
    pub fn from_int(v: u8) -> Option<InlineSignatureType> {
        Some(match v {
            0 => InlineSignatureType::Empty,
            1 => InlineSignatureType::I32,
            2 => InlineSignatureType::I64,
            3 => InlineSignatureType::F32,
            4 => InlineSignatureType::F64,
            _ => return None
        })
    }
}
