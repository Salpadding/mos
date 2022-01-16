use core::fmt::Arguments;

pub struct SliceWriter<'a> {
    buf: &'a mut [u8],
    off: usize,
}

impl<'a> SliceWriter<'a> {
   pub fn new(buf: &'a mut [u8]) -> Self {
       Self {
           buf,
           off: 0
       }
   }
}

impl<'a> core::fmt::Write for SliceWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let min = (self.buf.len() - self.off).min(s.as_bytes().len());
        self.buf[self.off..self.off + min].copy_from_slice(&s.as_bytes()[..min]);
        self.off += min;
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.buf[self.off] = c as u8;
        self.off += 1;
        Ok(())
    }
}
