pub const PRG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHR_ROM_UNIT_SIZE: usize = 0x2000;
pub const PRG_RAM_UNIT_SIZE: usize = 0x2000;

pub const CPU_CYCLES_PER_FRAME: u16 = 29780;

pub const DEFAULT_FPS: u16 = 60;
pub const DOUBLE_FPS: u16 = 120;
pub const HALF_FPS: u16 = 30;

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLowerBank,
    SingleScreenUpperBank,
}

pub fn convert_2u8_to_u16(b0: u8, b1: u8) -> u16 {
    (b0 as u16) | ((b1 as u16) << 8)
}

pub struct NonNullPtr<T: ?Sized>(std::ptr::NonNull<T>);
impl<T> NonNullPtr<T> {
    pub fn as_ref<'a>(&self) -> &'a T {
        unsafe { self.0.as_ref() }
    }
    pub fn as_mut<'a>(&mut self) -> &'a mut T {
        unsafe { self.0.as_mut() }
    }

    pub fn from(reference: &T) -> Self {
        Self(std::ptr::NonNull::from(reference))
    }
}

impl<T> Default for NonNullPtr<T> {
    fn default() -> Self {
        Self(std::ptr::NonNull::dangling())
    }
}

impl<T> Clone for NonNullPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for NonNullPtr<T> {}
