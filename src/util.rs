pub(crate) trait BitsExt {
    fn set(self, shift: u8) -> Self;
    fn clear(self, shift: u8) -> Self;
    fn check(self, shift: u8) -> Self;
    fn set_mask(self, mask: Self) -> Self;
    fn clear_mask(self, mask: Self) -> Self;
}

impl BitsExt for u8 {
    #[inline(always)]
    fn set(self, shift: u8) -> Self {
        self | (1 << shift)
    }

    #[inline(always)]
    fn clear(self, shift: u8) -> Self {
        self & !(1 << shift)
    }

    #[inline(always)]
    fn check(self, shift: u8) -> Self {
        self & (1 << shift)
    }

    #[inline(always)]
    fn set_mask(self, mask: Self) -> Self {
        self | mask
    }

    #[inline(always)]
    fn clear_mask(self, mask: Self) -> Self {
        self & !mask
    }
}

// Like Default, except it's private so HAL user's can't call it, preventing users from creating
// HAL objects out of thin air
pub trait SealedDefault {
    fn default() -> Self;
}
