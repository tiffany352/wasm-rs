use super::*;

pub struct MemorySection<'a> {
    pub count: u32,
    pub entries_raw: &'a [u8],
}

pub struct MemoryEntryIterator<'a> {
    count: u32,
    iter: &'a [u8]
}

pub struct MemoryEntry {
    pub limits: ResizableLimits,
}

impl<'a> MemorySection<'a> {
    pub fn entries(&self) -> MemoryEntryIterator<'a> {
        MemoryEntryIterator {
            count: self.count,
            iter: self.entries_raw
        }
    }
}

impl<'a> Iterator for MemoryEntryIterator<'a> {
    type Item = Result<MemoryEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None
        }
        self.count -= 1;
        let limits = try_opt!(ResizableLimits::parse(&mut self.iter));
        Some(Ok(MemoryEntry {
            limits: limits,
        }))
    }
}
