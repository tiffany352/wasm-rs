use super::*;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};

pub struct OpIterator<'a> {
    pub iter: &'a [u8],
    nesting: usize
}

#[derive(Debug, Clone)]
pub struct BrTable<'a> {
    pub count: u32,
    raw: &'a [u8],
    pub default: u32,
}

#[derive(Debug, Clone)]
pub struct BrTableArmIterator<'a> {
    count: u32,
    iter: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct MemoryImmediate {
    pub flags: u32,
    pub offset: u32
}

macro_rules! op_map {
    ($name:ident = $code:expr) => {
        Some(Op::$name)
    };
    ($name:ident $(( $($def:tt)+ ))+ = $code:expr) => {
        None
    }
}

macro_rules! optable {
    (Lifetime = $lifetime:tt, $($name:ident $(( $($def:tt)+ ))* = $code:expr),+) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum OpCode {$(
            $name = $code
        ),+}

        impl OpCode {
            pub fn from_int(v: u8) -> Option<OpCode> {
                match v {$(
                    $code => Some(OpCode::$name)
                ),+,
                         _ => None
                }
            }

            pub fn to_op<'a>(self) -> Option<Op<'a>> {
                match self {$(
                    OpCode::$name => op_map!($name $(($($def)+))* = $code)
                ),+}
            }
        }

        #[derive(Clone, Debug)]
        pub enum Op<$lifetime> {$(
            $name $(($($def)+))*
        ),+}
    }
}

optable! {
    Lifetime = 'a,

    // control flow
    Unreachable = 0x00,
    Nop = 0x01,
    Block(InlineSignatureType) = 0x02,
    Loop(InlineSignatureType) = 0x03,
    If(InlineSignatureType) = 0x04,
    Else = 0x05,
    End = 0x0b,
    Br(u32) = 0x0c,
    BrIf(u32) = 0x0d,
    BrTable(BrTable<'a>) = 0x0e,
    Return = 0x0f,

    // call operators
    Call(u32) = 0x10,
    CallIndirect(u32, bool) = 0x11,

    // parametric operators
    Drop = 0x1a,
    Select = 0x1b,

    // basic operators
    GetLocal(u32) = 0x20,
    SetLocal(u32) = 0x21,
    TeeLocal(u32) = 0x22,
    GetGlobal(u32) = 0x23,
    SetGlobal(u32) = 0x24,

    // memory-related
    I32Load(MemoryImmediate) = 0x28,
    I64Load(MemoryImmediate) = 0x29,
    F32Load(MemoryImmediate) = 0x2a,
    F64Load(MemoryImmediate) = 0x2b,
    I32Load8S(MemoryImmediate) = 0x2c,
    I32Load8U(MemoryImmediate) = 0x2d,
    I32Load16S(MemoryImmediate) = 0x2e,
    I32Load16U(MemoryImmediate) = 0x2f,
    I64Load8S(MemoryImmediate) = 0x30,
    I64Load8U(MemoryImmediate) = 0x31,
    I64Load16S(MemoryImmediate) = 0x32,
    I64Load16U(MemoryImmediate) = 0x33,
    I64Load32S(MemoryImmediate) = 0x34,
    I64Load32U(MemoryImmediate) = 0x35,
    I32Store(MemoryImmediate) = 0x36,
    I64Store(MemoryImmediate) = 0x37,
    F32Store(MemoryImmediate) = 0x38,
    F64Store(MemoryImmediate) = 0x39,
    I32Store8(MemoryImmediate) = 0x3a,
    I32Store16(MemoryImmediate) = 0x3b,
    I64Store8(MemoryImmediate) = 0x3c,
    I64Store16(MemoryImmediate) = 0x3d,
    I64Store32(MemoryImmediate) = 0x3e,
    CurrentMemory(bool) = 0x3f,
    GrowMemory(bool) = 0x40,

    // constants
    I32Const(i32) = 0x41,
    I64Const(i64) = 0x42,
    F32Const(f32) = 0x43,
    F64Const(f64) = 0x44,

    // comparison operators
    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4a,
    I32GtU = 0x4b,
    I32LeS = 0x4c,
    I32LeU = 0x4d,
    I32GeS = 0x4e,
    I32GeU = 0x4f,
    I64Eqz = 0x50,
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5a,
    F32Eq = 0x5b,
    F32Ne = 0x5c,
    F32Lt = 0x5d,
    F32Gt = 0x5e,
    F32Le = 0x5f,
    F32Ge = 0x60,
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    // numeric operators
    I32Clz = 0x67,
    I32Ctz = 0x68,
    I32Popcnt = 0x69,
    I32Add = 0x6a,
    I32Sub = 0x6b,
    I32Mul = 0x6c,
    I32DivS = 0x6d,
    I32DivU = 0x6e,
    I32RemS = 0x6f,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,
    I32Rotl = 0x77,
    I32Rotr = 0x78,
    I64Clz = 0x79,
    I64Ctz = 0x7a,
    I64Popcnt = 0x7b,
    I64Add = 0x7c,
    I64Sub = 0x7d,
    I64Mul = 0x7e,
    I64DivS = 0x7f,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,
    I64Rotl = 0x89,
    I64Rotr = 0x8a,
    F32Abs = 0x8b,
    F32Neg = 0x8c,
    F32Ceil = 0x8d,
    F32Floor = 0x8e,
    F32Trunc = 0x8f,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,
    F32Copysign = 0x98,
    F64Abs = 0x99,
    F64Neg = 0x9a,
    F64Ceil = 0x9b,
    F64Floor = 0x9c,
    F64Trunc = 0x9d,
    F64Nearest = 0x9e,
    F64Sqrt = 0x9f,
    F64Add = 0xa0,
    F64Sub = 0xa1,
    F64Mul = 0xa2,
    F64Div = 0xa3,
    F64Min = 0xa4,
    F64Max = 0xa5,
    F64Copysign = 0xa6,

    // conversions
    I32WrapI64 = 0xa7,
    I32TruncSF32 = 0xa8,
    I32TruncUF32 = 0xa9,
    I32TruncSF64 = 0xaa,
    I32TruncUF64 = 0xab,
    I64ExtendSI32 = 0xac,
    I64ExtendUI32 = 0xad,
    I64TruncSF32 = 0xae,
    I64TruncUF32 = 0xaf,
    I64TruncSF64 = 0xb0,
    I64TruncUF64 = 0xb1,
    F32ConvertSI32 = 0xb2,
    F32ConvertUI32 = 0xb3,
    F32ConvertSI64 = 0xb4,
    F32ConvertUI64 = 0xb5,
    F32DemoteF64 = 0xb6,
    F64ConvertSI32 = 0xb7,
    F64ConvertUI32 = 0xb8,
    F64ConvertSI64 = 0xb9,
    F64ConvertUI64 = 0xba,
    F64PromoteF32 = 0xbb,

    // reinterpretations
    I32ReinterpretF32 = 0xbc,
    I64ReinterpretF64 = 0xbd,
    F32ReinterpretI32 = 0xbe,
    F64ReinterpretI64 = 0xbf
}

impl MemoryImmediate {
    pub fn read(mut iter: &mut &[u8]) -> Result<MemoryImmediate, Error> {
        let flags = try!(read_varuint(iter));
        let offset = try!(read_varuint(iter));
        Ok(MemoryImmediate {
            flags: flags as u32,
            offset: offset as u32,
        })
    }
}

impl<'a> OpIterator<'a> {
    pub fn new(data: &'a [u8]) -> OpIterator<'a> {
        OpIterator {
            iter: data,
            nesting: 1
        }
    }
}

impl<'a> Iterator for OpIterator<'a> {
    type Item = Result<Op<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nesting == 0 {
            return None
        }
        let mut code = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut code));
        let code = try_opt!(OpCode::from_int(code[0]).ok_or(Error::UnknownVariant("opcode")));
        Some(Ok(match code {
            OpCode::Block => {
                self.nesting += 1;
                let mut sig = [0; 1];
                try_opt!((&mut self.iter).read_exact(&mut sig));
                let sig = try_opt!(InlineSignatureType::from_int(
                    sig[0]
                ).ok_or(Error::UnknownVariant("inline signature type")));
                Op::Block(sig)
            },
            OpCode::Loop => {
                self.nesting += 1;
                let mut sig = [0; 1];
                try_opt!((&mut self.iter).read_exact(&mut sig));
                let sig = try_opt!(InlineSignatureType::from_int(
                    sig[0]
                ).ok_or(Error::UnknownVariant("inline signature type")));
                Op::Loop(sig)
            },
            OpCode::If => {
                self.nesting += 1;
                let mut sig = [0; 1];
                try_opt!((&mut self.iter).read_exact(&mut sig));
                let sig = try_opt!(InlineSignatureType::from_int(
                    sig[0]
                ).ok_or(Error::UnknownVariant("inline signature type")));
                Op::If(sig)
            },
            OpCode::Br => {
                let depth = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::Br(depth)
            },
            OpCode::BrIf => {
                let depth = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::BrIf(depth)
            },
            OpCode::BrTable => {
                let count = try_opt!(read_varuint(&mut self.iter)) as u32;
                let start = self.iter;
                let len = self.iter.len();
                for _i in 0..count {
                    try_opt!(read_varuint(&mut self.iter));
                }
                let start = &start[..len - self.iter.len()];
                let default = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::BrTable(BrTable {
                    count: count,
                    raw: start,
                    default: default
                })
            },
            OpCode::I32Const => {
                let value = try_opt!(read_varint(&mut self.iter)) as i32;
                Op::I32Const(value)
            },
            OpCode::I64Const => {
                let value = try_opt!(read_varint(&mut self.iter)) as i64;
                Op::I64Const(value)
            },
            OpCode::F64Const => {
                let value = try_opt!((&mut self.iter).read_f64::<LittleEndian>());
                Op::F64Const(value)
            },
            OpCode::F32Const => {
                let value = try_opt!((&mut self.iter).read_f32::<LittleEndian>());
                Op::F32Const(value)
            },
            OpCode::GetLocal => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::GetLocal(value)
            },
            OpCode::SetLocal => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::SetLocal(value)
            },
            OpCode::TeeLocal => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::TeeLocal(value)
            },
            OpCode::GetGlobal => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::GetGlobal(value)
            },
            OpCode::SetGlobal => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::SetGlobal(value)
            },
            OpCode::Call => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                Op::Call(value)
            },
            OpCode::CallIndirect => {
                let value = try_opt!(read_varuint(&mut self.iter)) as u32;
                let reserved = try_opt!(read_varuint(&mut self.iter)) != 0;
                Op::CallIndirect(value, reserved)
            },
            OpCode::I32Load8S => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Load8S(value)
            },
            OpCode::I32Load8U => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Load8U(value)
            },
            OpCode::I32Load16S => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Load16S(value)
            },
            OpCode::I32Load16U => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Load16U(value)
            },
            OpCode::I64Load8S => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load8S(value)
            },
            OpCode::I64Load8U => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load8U(value)
            },
            OpCode::I64Load16S => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load16S(value)
            },
            OpCode::I64Load16U => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load16U(value)
            },
            OpCode::I64Load32S => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load32S(value)
            },
            OpCode::I64Load32U => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load32U(value)
            },
            OpCode::I32Load => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Load(value)
            },
            OpCode::I64Load => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Load(value)
            },
            OpCode::F32Load => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::F32Load(value)
            },
            OpCode::F64Load => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::F64Load(value)
            },
            OpCode::I32Store8 => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Store8(value)
            },
            OpCode::I32Store16 => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Store16(value)
            },
            OpCode::I64Store8 => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Store8(value)
            },
            OpCode::I64Store16 => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Store16(value)
            },
            OpCode::I64Store32 => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Store32(value)
            },
            OpCode::I32Store => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I32Store(value)
            },
            OpCode::I64Store => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::I64Store(value)
            },
            OpCode::F32Store => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::F32Store(value)
            },
            OpCode::F64Store => {
                let value = try_opt!(MemoryImmediate::read(&mut self.iter));
                Op::F64Store(value)
            },
            OpCode::End => {
                self.nesting -= 1;
                Op::End
            },
            x => x.to_op().expect("Missing case in instruction decoder")
        }))
    }
}

impl<'a> BrTable<'a> {
    pub fn arms(&self) -> BrTableArmIterator<'a> {
        BrTableArmIterator {
            count: self.count,
            iter: self.raw
        }
    }
}

impl<'a> Iterator for BrTableArmIterator<'a> {
    type Item = Result<u32, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        Some(Ok(try_opt!(read_varuint(&mut self.iter)) as u32))
    }
}
