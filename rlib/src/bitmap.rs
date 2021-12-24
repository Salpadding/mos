/// view [u8] as [bool]
pub trait Bitmap {
    fn init(&mut self);
    fn test(&self, bit_i: usize) -> bool;
    fn try_alloc(&self, bits: usize) -> isize;
    fn set(&mut self, bit_i: usize, v: bool);
    fn bits(&self) -> usize;
    fn fill_n(&mut self, start_bit: usize, n: usize);
}

impl Bitmap for [u8] {
    fn init(&mut self) {
        for x in self.iter_mut() { *x = 0; }
    }

    fn test(&self, bit_i: usize) -> bool {
        let j = bit_i / 8;
        let k = bit_i % 8;
        self[j] & (1 << k) != 0
    }

    // try to fill the bit map with continuous true
    // return the bit index if success, -1 if failed (no space)
    fn try_alloc(&self, bits: usize) -> isize {
        let mut byte_i: usize = 0;

        // find the first free byte
        while 0xff == self[byte_i] && byte_i < self.len() {
            byte_i += 1;
        }

        // find the first free bit
        if byte_i >= self.len() { return -1; }

        let mut bit_i = 0;
        for i in 0..8 {
            if (self[byte_i] & (1 << i)) == 0 {
                bit_i = i;
                break;
            }
        };

        if bits == 1 {
            return (byte_i * 8 + bit_i) as isize;
        }

        let mut cnt: usize = 0;

        // loop until last bit
        for i in (byte_i * 8 + bit_i)..self.bits() {
            if self.test(i) {
                cnt = 0;
            } else {
                cnt += 1;
            }

            if cnt == bits {
                return (i + 1 - bits) as isize;
            }
        }
        -1
    }

    fn set(&mut self, bit_i: usize, v: bool) {
        let j = bit_i / 8;
        let k = bit_i % 8;

        let msk: u8 = if v { 1 << k } else { !(1 << k) };
        self[j] = if v { self[j] | msk } else { self[j] & msk };
    }

    fn bits(&self) -> usize {
        self.len() * 8
    }

    fn fill_n(&mut self, start_bit: usize, n: usize) {
        let end = start_bit + n;
        let x = start_bit / 8 + 1;
        let y = end / 8;

        if y >= x {
            for i in x..y {
                self[i] = 0xff;
            }

            self[x - 1] = self[x - 1] |  (0xff << (start_bit % 8));

            if end % 8 > 0 {
                self[y] = self[y] |  (0xff >> (8 - end % 8));
            }
            return
        }

        if y < x {
            self[x - 1] = fill_byte(self[x - 1], start_bit % 8, n);
        }
    }
}

fn fill_byte(x: u8, i: usize, n: usize) -> u8 {
    x | ((0xff << i) & !(0xff << (i + n)))
}


#[cfg(test)]
mod test {
    use crate::bitmap::{Bitmap, fill_byte};

    #[test]
    fn test() {
        println!("testing!");
        use super::Bitmap;

        let mut x: Vec<u8> = vec![1 << 3, 0, 1 << 4];
        let start = x.try_alloc(4 + 8 + 3);
        println!("start = {}", start);
        x.fill_n(4, 15);

        for i in 0..x.bits() {
            println!("bit {} = {}", i, x.test(i));
        }
    }

    #[test]
    fn test1() {
        let mut x: [u8; 4] = [0, 0, 0, 0];
        let mut y = x.clone();
        let mut z = x.clone();

        x.fill_n(13, 10);
        y.fill_n(13, 11);
        z.fill_n(13, 7);

        for a in [x, y, z] {
            let s: Vec<_> = a.iter().map(|x| format!("{:08b}", x)).collect();
            println!("{:?}", s);
        }
    }
}
