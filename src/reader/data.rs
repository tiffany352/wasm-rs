use super::*;
use super::bytecode::{Op, OpIterator};

pub struct DataSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct DataEntryIterator<'a> {
    count: u32,
    opiter: Option<OpIterator<'a>>,
    iter: &'a [u8]
}

pub enum DataEntry<'a> {
    Index(u32),
    Op(Op<'a>),
    Data(&'a [u8]),
}

impl<'a> DataSection<'a> {
    pub fn entries(&self) -> DataEntryIterator<'a> {
        DataEntryIterator {
            count: self.count,
            opiter: None,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for DataEntryIterator<'a> {
    type Item = Result<DataEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        if let Some(mut iter) = self.opiter.take() {
            if let Some(op) = iter.next() {
                self.opiter = Some(iter);
                return Some(op.map(DataEntry::Op))
            }
            let size = try_opt!(read_varuint(&mut self.iter)) as usize;
            let res = &self.iter[..size];
            self.iter = &self.iter[size..];
            return Some(Ok(DataEntry::Data(res)))
        }
        let index = try_opt!(read_varuint(&mut self.iter)) as u32;
        Some(Ok(DataEntry::Index(index)))
    }
}
