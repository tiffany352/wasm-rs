use super::*;
use std::io::Read;
use super::bytecode::{Op, OpIterator};

pub struct GlobalSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct GlobalEntryIterator<'a> {
    count: u32,
    opiter: Option<OpIterator<'a>>,
    iter: &'a [u8]
}

pub struct GlobalEntry {
    pub ty: ValueType,
    pub mutable: bool,
}

pub enum GlobalEntryEither<'a> {
    Entry(GlobalEntry),
    Op(Op<'a>),
}

impl<'a> GlobalSection<'a> {
    pub fn entries(&self) -> GlobalEntryIterator<'a> {
        GlobalEntryIterator {
            count: self.count,
            opiter: None,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for GlobalEntryIterator<'a> {
    type Item = Result<GlobalEntryEither<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut iter) = self.opiter.take() {
            if let Some(op) = iter.next() {
                self.iter = iter.iter;
                self.opiter = Some(iter);
                return Some(op.map(GlobalEntryEither::Op))
            }
        }
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let mut ty = [0; 1];
        try_opt!((&mut self.iter).read_exact(&mut ty));
        let mutable = try_opt!(read_varuint(&mut self.iter)) != 0;
        self.opiter = Some(OpIterator::new(self.iter));
        Some(Ok(GlobalEntryEither::Entry(GlobalEntry {
            ty: try_opt!(ValueType::from_int(ty[0]).ok_or(Error::UnknownVariant("value type"))),
            mutable: mutable,
        })))
    }
}
