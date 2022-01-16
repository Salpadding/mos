use core::fmt::Arguments;

pub struct SliceWriter<'a>(pub &'a mut [u8]);

impl<'a> core::fmt::Write for SliceWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let min = self.0.len().min(s.as_bytes().len());
        self.0[..min].copy_from_slice(&s.as_bytes()[..min]);
        Ok(())
    }
}
