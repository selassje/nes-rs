use thiserror::Error;
/* 
pub struct NesRomTooShort {
    pub expected_size: usize,
    pub actual_size: usize,
}

impl Display for NesRomTooShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NES ROM is too short. Expected at least {} bytes, but got {} bytes.",
            self.expected_size, self.actual_size
        )
    }
}

pub struct UnknownNesFormat {}

impl Display for UnknownNesFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown NES file format detected.")
    }
}
*/
#[derive(Error, Debug)]
pub enum Error {
    #[error("NES ROM is too short. Expected at least {0} bytes, but got {1} bytes.")]
    NesRomTooShort(usize,usize),
    #[error("Unknown NES file format detected.")]
    UnknownNesFormat,
}
