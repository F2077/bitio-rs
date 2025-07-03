use crate::error::BitReadWriteError;
use std::io::{Read, Result};

/// Ultra-fast bit reader for BigEndian streams (~18x faster than standard)
///
/// ## Critical Performance Notice
/// - This implementation does **NOT** implement the standard `BitRead` trait to avoid abstraction overhead and enable aggressive optimizations.
/// - API **intentionally** differs from standard implementations to ensure users are clearly aware they are using an incompatible version.
///
/// ## Performance Characteristics (Mac mini M4 16GB benchmark)
/// - Reading 32 bits: ~290 ns (nanoseconds)
/// - ~18x faster than standard `BitReader`
///
/// ⚠️ **Use at your own risk**
pub struct FastBitReaderBig<R: Read> {
    raw: R,
    buffer: u64,
    bits_available: usize,
    scratch: [u8; 8],
}

impl<R: Read> FastBitReaderBig<R> {
    #[inline]
    pub fn new(raw: R) -> Self {
        Self {
            raw,
            buffer: 0,
            bits_available: 0,
            scratch: [0; 8],
        }
    }

    /// Reads 1..=64 bits with maximal performance
    #[inline(always)]
    pub fn read_bits_fast(&mut self, n: usize) -> Result<u64> {
        if n == 0 || n > 64 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }

        while self.bits_available < n {
            let remaining_bits = n - self.bits_available;
            let max_bytes = (64 - self.bits_available) / 8;
            let needed_bytes = ((remaining_bits + 7) / 8).min(max_bytes).max(1);

            self.raw.read_exact(&mut self.scratch[..needed_bytes])?;

            let mut val = 0u64;
            for i in 0..needed_bytes {
                val = (val << 8) | self.scratch[i] as u64;
            }

            let new_bits = needed_bytes * 8;
            let shift = 64 - self.bits_available - new_bits;
            self.buffer |= val.wrapping_shl(shift as u32);
            self.bits_available += new_bits;
        }

        let result = self.buffer >> (64 - n);
        // 对 n==64 做特殊处理，避免溢出
        if n < 64 {
            self.buffer <<= n;
        } else {
            self.buffer = 0;
        }
        self.bits_available -= n;
        Ok(result)
    }
}

/// Ultra-fast bit reader for LittleEndian streams (~21x faster than standard)
///
/// ## Critical Performance Notice
/// - This implementation does **NOT** implement the standard `BitRead` trait to avoid abstraction overhead and enable aggressive optimizations.
/// - API **intentionally** differs from standard implementations to ensure users are clearly aware they are using an incompatible version.
///
/// ## Performance Characteristics (Mac mini M4 16GB benchmark)
/// - Reading 32 bits: ~260 ns (nanoseconds)
/// - ~21x faster than standard `BitReader`
///
/// ⚠️ **Use at your own risk**
pub struct FastBitReaderLittle<R: Read> {
    raw: R,
    buffer: u64,
    bits_available: usize,
    scratch: [u8; 8],
}

impl<R: Read> FastBitReaderLittle<R> {
    #[inline]
    pub fn new(raw: R) -> Self {
        Self {
            raw,
            buffer: 0,
            bits_available: 0,
            scratch: [0; 8],
        }
    }

    /// Reads bits with extreme performance (0-64 bits)
    ///
    /// Same performance characteristics and safety considerations
    /// as `FastBitReaderBig::read_bits_fast` but for LittleEndian data.
    #[inline(always)]
    pub fn read_bits_fast(&mut self, n: usize) -> Result<u64> {
        if n == 0 || n > 64 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }

        while self.bits_available < n {
            let remaining_bits = n - self.bits_available;
            let max_bytes = (64 - self.bits_available) / 8;
            let needed_bytes = ((remaining_bits + 7) / 8).min(max_bytes).max(1);

            self.raw.read_exact(&mut self.scratch[..needed_bytes])?;

            let mut val = 0u64;
            for i in 0..needed_bytes {
                val |= (self.scratch[i] as u64) << (i * 8);
            }

            let new_bits = needed_bytes * 8;
            self.buffer |= val.wrapping_shl(self.bits_available as u32);
            self.bits_available += new_bits;
        }

        let mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
        let result = self.buffer & mask;
        // 对 n==64 做特殊处理，避免溢出
        if n < 64 {
            self.buffer >>= n;
        } else {
            self.buffer = 0;
        }
        self.bits_available -= n;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ================ Big Endian 测试 ================
    #[test]
    fn test_big_endian_basic() {
        let data = [0b1010_1010, 0b1100_1100];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits_fast(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits_fast(8).unwrap(), 0b1100_1100);
    }

    #[test]
    fn test_big_endian_cross_byte() {
        let data = [0b1100_1100, 0b1010_1010];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(3).unwrap(), 0b110);
        assert_eq!(reader.read_bits_fast(10).unwrap(), 0b0_11001010_1);
    }

    #[test]
    fn test_big_endian_large_read() {
        let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(64).unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn test_big_endian_multiple_fills() {
        let data = [0xFF; 16];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(56).unwrap(), 0xFFFFFFFFFFFFFF);
        assert_eq!(reader.read_bits_fast(64).unwrap(), 0xFFFFFFFFFFFFFFFF);
    }

    // ================ Little Endian 测试 ================
    #[test]
    fn test_little_endian_basic() {
        let data = [0b1010_1010, 0b1100_1100];
        let mut reader = FastBitReaderLittle::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits_fast(4).unwrap(), 0b1010);
        assert_eq!(reader.read_bits_fast(8).unwrap(), 0b1100_1100);
    }

    #[test]
    fn test_little_endian_cross_byte() {
        let data = [0b0000_0001, 0b1000_0000];
        let mut reader = FastBitReaderLittle::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(1).unwrap(), 1);
        assert_eq!(reader.read_bits_fast(7).unwrap(), 0);
        assert_eq!(reader.read_bits_fast(1).unwrap(), 0);
        assert_eq!(reader.read_bits_fast(7).unwrap(), 0b1000000);
    }

    #[test]
    fn test_little_endian_large_read() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut reader = FastBitReaderLittle::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(64).unwrap(), 0x0807060504030201);
    }

    #[test]
    fn test_little_endian_multiple_fills() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        let mut reader = FastBitReaderLittle::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(56).unwrap(), 0x07060504030201);
        assert_eq!(reader.read_bits_fast(16).unwrap(), 0x0908);
    }

    // ================ 通用边界测试 ================
    #[test]
    fn test_read_past_end() {
        let data = [0x12, 0x34];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        assert_eq!(reader.read_bits_fast(16).unwrap(), 0x1234);
        assert!(reader.read_bits_fast(1).is_err());
    }

    #[test]
    fn test_zero_bits() {
        let data = [0xAA];
        let mut reader = FastBitReaderLittle::new(Cursor::new(data));
        assert!(reader.read_bits_fast(0).is_err());
        assert_eq!(reader.read_bits_fast(8).unwrap(), 0xAA);
    }

    #[test]
    fn test_read_more_than_64_bits() {
        let data = [0xFF; 16];
        let mut reader = FastBitReaderBig::new(Cursor::new(data));
        let result = reader.read_bits_fast(65);
        assert!(result.is_err());
    }
}
