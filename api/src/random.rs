#[cfg(target_os = "linux")]
use std::ffi::c_uint;
use std::ops::Range;

unsafe extern "C" {
    #[cfg(target_os = "linux")]
    fn getrandom(buf: *mut u8, len: usize, flags: c_uint) -> isize;
    #[cfg(target_os = "windows")]
    #[link_name = "ProcessPrng"]
    fn process_prng(buf: *mut u8, len: usize) -> bool;
}

pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    pub fn new() -> Self {
        let mut state = [0; 4];
        #[cfg(target_os = "linux")]
        unsafe {
            let bytes = getrandom((&raw mut state).cast(), size_of_val(&state), 0);
            if bytes != size_of_val(&state) as isize {
                panic!("Cannot get a random value.");
            }
        }
        #[cfg(target_os = "windows")]
        unsafe {
            process_prng((&raw mut state).cast(), size_of_val(&state));
        }
        Self { state }
    }

    #[inline]
    fn next(&mut self) -> u64 {
        let result = self.state[0]
            .rotate_left(23)
            .wrapping_add(self.state[3])
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;

        self.state[3] = self.state[3].rotate_left(45);

        result
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_int {
    ($func:ident, $ty:ty) => {
        pub fn $func(&mut self) -> $ty {
            self.next() as $ty
        }
    };
}

macro_rules! impl_unsigned_range {
    ($func:ident, $ty:ty, $raw:ident) => {
        pub fn $func(&mut self, range: Range<$ty>) -> $ty {
            assert!(
                range.start < range.end,
                "The random range must not be empty."
            );
            let length = range.end - range.start;
            let limit = <$ty>::MAX - (<$ty>::MAX % length);
            loop {
                let value = self.$raw();
                if value < limit {
                    return range.start + value % length;
                }
            }
        }
    };
}

macro_rules! impl_signed_range {
    ($func:ident, $ty:ty, $raw:ident, $raw_ty:ty) => {
        pub fn $func(&mut self, range: Range<$ty>) -> $ty {
            assert!(
                range.start < range.end,
                "The random range must not be empty."
            );
            let length = (range.end as i128 - range.start as i128) as u128;
            let max = <$raw_ty>::MAX as u128;
            let limit = max - max % length;
            loop {
                let value = self.$raw() as u128;
                if value < limit {
                    return (range.start as i128 + (value % length) as i128) as $ty;
                }
            }
        }
    };
}

impl Rng {
    pub fn get_bool(&mut self) -> bool {
        self.next() & 1 == 1
    }

    impl_unsigned_range!(get_u8_range, u8, get_u8);
    impl_unsigned_range!(get_u16_range, u16, get_u16);
    impl_unsigned_range!(get_u32_range, u32, get_u32);
    impl_unsigned_range!(get_u64_range, u64, get_u64);
    impl_unsigned_range!(get_usize_range, usize, get_usize);

    impl_signed_range!(get_i8_range, i8, get_u8, u8);
    impl_signed_range!(get_i16_range, i16, get_u16, u16);
    impl_signed_range!(get_i32_range, i32, get_u32, u32);
    impl_signed_range!(get_i64_range, i64, get_u64, u64);
    impl_signed_range!(get_isize_range, isize, get_usize, usize);

    impl_int!(get_u8, u8);
    impl_int!(get_u16, u16);
    impl_int!(get_u32, u32);
    impl_int!(get_u64, u64);
    impl_int!(get_usize, usize);

    impl_int!(get_i8, i8);
    impl_int!(get_i16, i16);
    impl_int!(get_i32, i32);
    impl_int!(get_i64, i64);
    impl_int!(get_isize, isize);
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn range_values_stay_inside_bounds() {
        let mut rng = Rng::new();
        for _ in 0..500 {
            assert!((7..12).contains(&rng.get_u8_range(7..12)));
            assert!((-8..-2).contains(&rng.get_i32_range(-8..-2)));
        }
    }

    #[test]
    #[should_panic(expected = "The random range must not be empty.")]
    fn empty_ranges_panic() {
        Rng::new().get_u8_range(4..4);
    }
}
