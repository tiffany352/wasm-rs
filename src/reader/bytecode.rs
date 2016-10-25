use super::*;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};

pub struct OpIterator<'a> {
    pub iter: &'a [u8],
    nesting: usize
}

pub struct BrTable<'a> {
    pub count: u32,
    raw: &'a [u8],
    pub default: u32,
}

pub struct BrTableArmIterator<'a> {
    count: u32,
    iter: &'a [u8],
}

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

        pub enum Op<$lifetime> {$(
            $name $(($($def)+))*
        ),+}
    }
}

optable! {
    Lifetime = 'a,

    // control flow
    Unreachable = 0x00,
    Block(InlineSignatureType) = 0x01,
    Loop(InlineSignatureType) = 0x02,
    If(InlineSignatureType) = 0x03,
    Else = 0x04,
    Select = 0x05,
    Br(u32) = 0x06,
    BrIf(u32) = 0x07,
    BrTable(BrTable<'a>) = 0x08,
    Return = 0x09,
    Drop = 0x0b,
    Nop = 0x0a,
    End = 0x0f,

    // basic operators
    I32Const(i32) = 0x10,
    I64Const(i64) = 0x11,
    F64Const(f64) = 0x12,
    F32Const(f32) = 0x13,
    GetLocal(u32) = 0x14,
    SetLocal(u32) = 0x15,
    TeeLocal(u32) = 0x19,
    GetGlobal(u32) = 0xbb,
    SetGlobal(u32) = 0xbc,
    Call(u32) = 0x16,
    CallIndirect(u32) = 0x17,

    // memory-related
    I32Load8S(MemoryImmediate) = 0x20,
    I32Load8U(MemoryImmediate) = 0x21,
    I32Load16S(MemoryImmediate) = 0x22,
    I32Load16U(MemoryImmediate) = 0x23,
    I64Load8S(MemoryImmediate) = 0x24,
    I64Load8U(MemoryImmediate) = 0x25,
    I64Load16S(MemoryImmediate) = 0x26,
    I64Load16U(MemoryImmediate) = 0x27,
    I64Load32S(MemoryImmediate) = 0x28,
    I64Load32U(MemoryImmediate) = 0x29,
    I32Load(MemoryImmediate) = 0x2a,
    I64Load(MemoryImmediate) = 0x2b,
    F32Load(MemoryImmediate) = 0x2c,
    F64Load(MemoryImmediate) = 0x2d,
    I32Store8(MemoryImmediate) = 0x2e,
    I32Store16(MemoryImmediate) = 0x2f,
    I64Store8(MemoryImmediate) = 0x30,
    I64Store16(MemoryImmediate) = 0x31,
    I64Store32(MemoryImmediate) = 0x32,
    I32Store(MemoryImmediate) = 0x33,
    I64Store(MemoryImmediate) = 0x34,
    F32Store(MemoryImmediate) = 0x35,
    F64Store(MemoryImmediate) = 0x36,
    CurrentMemory = 0x3b,
    GrowMemory = 0x39,

    I32Add = 0x40,
    I32Sub = 0x41,
    I32Mul = 0x42,
    I32DivS = 0x43,
    I32DivU = 0x44,
    I32RemS = 0x45,
    I32RemU = 0x46,
    I32And = 0x47,
    I32Or = 0x48,
    I32Xor = 0x49,
    I32Shl = 0x4a,
    I32ShrU = 0x4b,
    I32ShrS = 0x4c,
    I32Rotr = 0xb6,
    I32Rotl = 0xb7,
    I32Eq = 0x4d,
    I32Ne = 0x4e,
    I32LtS = 0x4f,
    I32LeS = 0x50,
    I32LtU = 0x51,
    I32LeU = 0x52,
    I32GtS = 0x53,
    I32GeS = 0x54,
    I32GtU = 0x55,
    I32GeU = 0x56,
    I32Clz = 0x57,
    I32Ctz = 0x58,
    I32Popcnt = 0x59,
    I32Eqz = 0x5a,
    I64Add = 0x5b,
    I64Sub = 0x5c,
    I64Mul = 0x5d,
    I64DivS = 0x5e,
    I64DivU = 0x5f,
    I64RemS = 0x60,
    I64RemU = 0x61,
    I64And = 0x62,
    I64Or = 0x63,
    I64Xor = 0x64,
    I64Shl = 0x65,
    I64ShrU = 0x66,
    I64ShrS = 0x67,
    I64Rotr = 0xb8,
    I64Rotl = 0xb9,
    I64Eq = 0x68,
    I64Ne = 0x69,
    I64LtS = 0x6a,
    I64LeS = 0x6b,
    I64LtU = 0x6c,
    I64LeU = 0x6d,
    I64GtS = 0x6e,
    I64GeS = 0x6f,
    I64GtU = 0x70,
    I64GeU = 0x71,
    I64Clz = 0x72,
    I64Ctz = 0x73,
    I64Popcnt = 0x74,
    I64Eqz = 0xba,
    F32Add = 0x75,
    F32Sub = 0x76,
    F32Mul = 0x77,
    F32Div = 0x78,
    F32Min = 0x79,
    F32Max = 0x7a,
    F32Abs = 0x7b,
    F32Neg = 0x7c,
    F32Copysign = 0x7d,
    F32Ceil = 0x7e,
    F32Floor = 0x7f,
    F32Trunc = 0x80,
    F32Nearest = 0x81,
    F32Sqrt = 0x82,
    F32Eq = 0x83,
    F32Ne = 0x84,
    F32Lt = 0x85,
    F32Le = 0x86,
    F32Gt = 0x87,
    F32Ge = 0x88,
    F64Add = 0x89,
    F64Sub = 0x8a,
    F64Mul = 0x8b,
    F64Div = 0x8c,
    F64Min = 0x8d,
    F64Max = 0x8e,
    F64Abs = 0x8f,
    F64Neg = 0x90,
    F64Copysign = 0x91,
    F64Ceil = 0x92,
    F64Floor = 0x93,
    F64Trunc = 0x94,
    F64Nearest = 0x95,
    F64Sqrt = 0x96,
    F64Eq = 0x97,
    F64Ne = 0x98,
    F64Lt = 0x99,
    F64Le = 0x9a,
    F64Gt = 0x9b,
    F64Ge = 0x9c,
    I32TruncSF32 = 0x9d,
    I32TruncSF64 = 0x9e,
    I32TruncUF32 = 0x9f,
    I32TruncUF64 = 0xa0,
    I32WrapI64 = 0xa1,
    I64TruncSF32 = 0xa2,
    I64TruncSF64 = 0xa3,
    I64TruncUF32 = 0xa4,
    I64TruncUF64 = 0xa5,
    I64ExtendSI32 = 0xa6,
    I64ExtendUI32 = 0xa7,
    F32ConvertSI32 = 0xa8,
    F32ConvertUI32 = 0xa9,
    F32ConvertSI64 = 0xaa,
    F32ConvertUI64 = 0xab,
    F32DemoteF64 = 0xac,
    F32ReinterpretI32 = 0xad,
    F64ConvertSI32 = 0xae,
    F64ConvertUI32 = 0xaf,
    F64ConvertSI64 = 0xb0,
    F64ConvertUI64 = 0xb1,
    F64PromoteF32 = 0xb2,
    F64ReinterpretI64 = 0xb3,
    I32ReinterpretF32 = 0xb4,
    I64ReinterpretF64 = 0xb5
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
                Op::CallIndirect(value)
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
