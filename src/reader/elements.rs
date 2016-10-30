use super::*;
use super::bytecode::{Op, OpIterator};

pub struct ElementSection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct ElementEntryIterator<'a> {
    count: u32,
    opiter: Option<OpIterator<'a>>,
    elems: usize,
    iter: &'a [u8]
}

pub enum ElementEntry<'a> {
    Index(u32),
    Op(Op<'a>),
    Elem(u32),
}

impl<'a> ElementSection<'a> {
    pub fn entries(&self) -> ElementEntryIterator<'a> {
        ElementEntryIterator {
            count: self.count,
            opiter: None,
            elems: 0,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for ElementEntryIterator<'a> {
    type Item = Result<ElementEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut iter) = self.opiter.take() {
            if let Some(op) = iter.next() {
                self.iter = iter.iter;
                self.opiter = Some(iter);
                return Some(op.map(ElementEntry::Op))
            }
            self.elems = try_opt!(read_varuint(&mut self.iter)) as usize;
        }
        if self.elems > 0 {
            self.elems -= 1;
            return Some(Ok(ElementEntry::Elem(try_opt!(
                read_varuint(&mut self.iter)
            ) as u32)));
        }
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let index = try_opt!(read_varuint(&mut self.iter)) as u32;
        self.opiter = Some(OpIterator::new(self.iter));
        Some(Ok(ElementEntry::Index(index)))
    }
}
