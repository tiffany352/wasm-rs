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
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
}

impl ValueType {
    pub fn from_int(v: u8) -> Option<ValueType> {
        Some(match v {
            0x7f => ValueType::I32,
            0x7e => ValueType::I64,
            0x7d => ValueType::F32,
            0x7c => ValueType::F64,
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
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    Empty = 0x40,
}

impl InlineSignatureType {
    pub fn from_int(v: u8) -> Option<InlineSignatureType> {
        Some(match v {
            0x7f => InlineSignatureType::I32,
            0x7e => InlineSignatureType::I64,
            0x7d => InlineSignatureType::F32,
            0x7c => InlineSignatureType::F64,
            0x40 => InlineSignatureType::Empty,
            _ => return None
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LanguageType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    AnyFunc = 0x70,
    Func = 0x60,
    /// Empty block type.
    Void = 0x40,
}

impl LanguageType {
    pub fn from_int(v: u8) -> Option<LanguageType> {
        Some(match v {
            0x7f => LanguageType::I32,
            0x7e => LanguageType::I64,
            0x7d => LanguageType::F32,
            0x7c => LanguageType::F64,
            0x70 => LanguageType::AnyFunc,
            0x60 => LanguageType::Func,
            0x40 => LanguageType::Void,
            _ => return None
        })
    }
}
