use super::*;

pub struct FunctionSection<'a>(pub &'a [u8], pub usize);

pub struct FunctionEntryIterator<'a>(&'a [u8], usize);

impl<'a> FunctionSection<'a> {
    pub fn types(&self) -> FunctionEntryIterator<'a> {
        FunctionEntryIterator(self.0, self.1)
    }
}

impl<'a> Iterator for FunctionEntryIterator<'a> {
    type Item = Result<u32, Error>;

    fn next(&mut self) -> Option<Result<u32, Error>> {
        if self.1 == 0 {
            return None
        }
        self.1 -= 1;
        Some(read_varuint(&mut self.0).map(|x| x as u32).map_err(|x| x.into()))
    }
}
