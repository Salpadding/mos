use rlib::bitmap::Bitmap;

pub struct Mem<'a> {
    m: &'a mut [u8]
}