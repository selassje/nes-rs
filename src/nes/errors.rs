use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("NES ROM is too short. Expected at least 16 bytes, but got {0} bytes.")]
    NesRomHeaderTooShort(usize),
    #[error("NES ROM Trainer too short. Expected at least 512 bytes, but got {0} bytes.")]
    NesRomTrainerTooShort(usize),
    #[error("NES PRG ROM unit {0} too short. Expected at least 16384 bytes, but got {1} bytes.")]
    NesPrgRomTooShort(u8, usize),
    #[error("NES CHR ROM unit {0} too short. Expected at least 8192 bytes, but got {1} bytes.")]
    NesChrRomTooShort(u8, usize),
    #[error(
        "NES ROM PlayChoice size is too short. Expected at least 8224 bytes, but got {0} bytes."
    )]
    NesPlayChoiceRomTooShort(usize),
    #[error("Unsupported Mapper {0}.")]
    NesUnsupportedMapper(u8),
    #[error("Unknown NES file format detected.")]
    UnknownNesFormat,
    #[error("Loaded state version mismatch. Expected version '{0}', but found version '{1}'.")]
    LoadStateVersionMismatch(String, String),
    #[error("Load state internal error {0}")]
    LoadStateInternalError(String),
    #[error("Save state internal error {0}")]
    SaveStateInternalError(String),
    #[error("Load state decompression  error {0:?}")]
    LoadStateDecompressionError(yazi::Error),
    #[error("Save state compression  error {0:?}")]
    LoadStateCompressionError(yazi::Error),
}
