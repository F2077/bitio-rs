pub trait BitRead {
    type Output;

    /// Reads exactly `n` bits, consuming them from the stream
    fn read_bits(&mut self, n: usize) -> std::io::Result<Self::Output>;
}

pub trait BitPeek {
    type Output;

    /// Peeks at the next `n` bits without consuming
    fn peek_bits(&mut self, n: usize) -> std::io::Result<Self::Output>;
}

pub trait BitWrite {
    fn write_bits(&mut self, value: u64, n: usize) -> std::io::Result<()>;
}
