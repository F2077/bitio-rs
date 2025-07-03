use std::fmt::Formatter;

#[derive(Debug)]
pub enum BitReadWriteError {
    InvalidBitCount(usize),
    UnexpectedEof,
    UnalignedAccess,
}

impl std::fmt::Display for BitReadWriteError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BitReadWriteError::InvalidBitCount(n) => {
                write!(f, "Bit count must be between 1-64, got {}", n)
            }
            BitReadWriteError::UnexpectedEof => write!(f, "Unexpected end of stream"),
            BitReadWriteError::UnalignedAccess => {
                write!(f, "Attempted to consume bytes while bits are buffered")
            }
        }
    }
}

impl std::error::Error for BitReadWriteError {}

impl From<BitReadWriteError> for std::io::Error {
    fn from(e: BitReadWriteError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}
