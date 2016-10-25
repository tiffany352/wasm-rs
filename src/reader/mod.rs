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

pub use leb128::read::unsigned as read_varuint;
pub use leb128::read::signed as read_varint;

pub mod enums;
pub use self::enums::{ValueType, SectionType, ExternalKind, InlineSignatureType};

pub mod types;
pub use self::types::{
    TypeSection, TypeEntryIterator, TypeEntry,
    FunctionType, ParamsIterator
};

pub mod imports;
pub use self::imports::{
    ImportSection, ImportEntryIterator, ImportEntry,
    ImportEntryContents, ResizableLimits
};

pub mod functions;
pub use self::functions::{
    FunctionSection, FunctionEntryIterator
};

pub mod modules;
pub use self::modules::{
    Module, SectionsIterator, Section, SectionContent
};

pub mod tables;
pub use self::tables::{
    TableSection, TableEntryIterator, TableEntry
};

pub mod memories;
pub use self::memories::{
    MemorySection, MemoryEntryIterator, MemoryEntry
};

pub mod globals;
pub use self::globals::{
    GlobalSection, GlobalEntryIterator, GlobalEntry
};

pub mod exports;
pub use self::exports::{
    ExportSection, ExportEntryIterator, ExportEntry
};

pub mod elements;
pub use self::elements::{
    ElementSection, ElementEntryIterator, ElementEntry
};

pub mod codes;
pub use self::codes::{
    CodeSection, CodeIterator, FunctionBody
};

pub mod data;
pub use self::data::{
    DataSection, DataEntryIterator, DataEntry
};

pub mod bytecode;
