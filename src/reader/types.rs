use super::*;
use std::io;

pub struct TypeSection<'a>(pub &'a [u8], pub usize);

pub struct TypeEntryIterator<'a>(&'a [u8], usize);

pub enum TypeEntry<'a> {
    Function(FunctionType<'a>),
}

pub struct FunctionType<'a> {
    params_count: usize,
    params_raw: &'a [u8],
    pub return_type: Option<ValueType>,
}

pub struct ParamsIterator<'a>(&'a [u8], usize);

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
