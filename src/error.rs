use std::fmt::Formatter;

#[derive(Debug)]
pub enum BitReaderError {
    InvalidBitCount(usize),
    UnexpectedEof,
}

impl std::fmt::Display for BitReaderError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BitReaderError::InvalidBitCount(n) => {
                write!(f, "Bit count must be between 1-64, got {}", n)
            }
            BitReaderError::UnexpectedEof => write!(f, "Unexpected end of stream"),
        }
    }
}

impl std::error::Error for BitReaderError {}

impl From<BitReaderError> for std::io::Error {
    fn from(e: BitReaderError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}
