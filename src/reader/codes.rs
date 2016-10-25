use super::*;
use std::io::Read;
use super::bytecode::{Op, OpIterator};

pub struct CodeSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct CodeIterator<'a> {
    count: u32,
    iter: &'a [u8]
}

pub struct FunctionBody<'a> {
    local_count: usize,
    body: &'a [u8],
}

pub struct FunctionIterator<'a> {
    local_count: usize,
    opiter: Option<OpIterator<'a>>,
    iter: &'a [u8],
}

pub enum FunctionPart<'a> {
    Local(Local),
    Op(Op<'a>),
}

pub struct Local {
    pub count: u32,
    pub ty: ValueType,
}

impl<'a> CodeSection<'a> {
    pub fn entries(&self) -> CodeIterator<'a> {
        CodeIterator {
            count: self.count,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for CodeIterator<'a> {
    type Item = Result<FunctionBody<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let body_size = try_opt!(read_varuint(&mut self.iter)) as usize;
        let local_count = try_opt!(read_varuint(&mut self.iter)) as usize;
        let body = {
            let res = &self.iter[..body_size];
            self.iter = &self.iter[body_size..];
            res
        };
        Some(Ok(FunctionBody {
            local_count: local_count,
            body: body,
        }))
    }
}

impl<'a> FunctionBody<'a> {
    pub fn contents(&self) -> FunctionIterator<'a> {
        FunctionIterator {
            local_count: self.local_count,
            opiter: None,
            iter: self.body
        }
    }
}

impl<'a> Iterator for FunctionIterator<'a> {
    type Item = Result<FunctionPart<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.local_count == 0 && self.opiter.is_none() {
            self.opiter = Some(OpIterator::new(self.iter))
        }
        if let Some(ref mut iter) = self.opiter {
            return iter.next().map(|x| x.map(FunctionPart::Op))
        }
        self.local_count -= 1;
        let count = try_opt!(read_varuint(&mut self.iter)) as u32;
        let mut ty = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut ty));
        let ty = try_opt!(ValueType::from_int(ty[0]).ok_or(Error::UnknownVariant("value type")));
        Some(Ok(FunctionPart::Local(Local {
            count: count,
            ty: ty,
        })))
    }
}
