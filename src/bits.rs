pub trait BitsExt {
    fn set(self, shift: u8) -> Self;
    fn clear(self, shift: u8) -> Self;
    fn check(self, shift: u8) -> Self;
    fn set_mask(self, mask: Self) -> Self;
    fn clear_mask(self, mask: Self) -> Self;
}

impl BitsExt for u8 {
    fn set(self, shift: u8) -> Self {
        self | (1 << shift)
    }

    fn clear(self, shift: u8) -> Self {
        self & !(1 << shift)
    }

    fn check(self, shift: u8) -> Self {
        self & (1 << shift)
    }

    fn set_mask(self, mask: Self) -> Self {
        self | mask
    }

    fn clear_mask(self, mask: Self) -> Self {
        self & !mask
    }
}
