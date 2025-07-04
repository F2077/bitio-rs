use crate::byte_order::ByteOrder;
use crate::error::BitReadWriteError;
use crate::traits::{BitPeek, BitRead};
use std::io::{BufReader, Read};

// ------------------------------- BitReader ------------------------------- //

pub struct BitReader<R: Read> {
    byte_order: ByteOrder,
    inner: BufReader<R>,

    bits_buffer: u64, // 比特缓冲区：rust 中并没有表达 "一系列比特" 的具名数据结构，但是事实上 u64 就可以表达一系列比特
    bits_in_buffer: usize, // 当前比特缓冲区中持有的比特数
}

impl<R: Read> BitReader<R> {
    pub fn new(inner: R) -> Self {
        Self::with_byte_order(ByteOrder::BigEndian, inner)
    }

    pub fn with_byte_order(byte_order: ByteOrder, inner: R) -> Self {
        Self {
            byte_order,
            inner: BufReader::new(inner),
            bits_buffer: 0,
            bits_in_buffer: 0,
        }
    }
}

impl<R: Read> BitReader<R> {
    fn put_into_bits_buffer(&mut self, n: usize) -> std::io::Result<()> {
        let bits_needed = n.saturating_sub(self.bits_in_buffer); // 使用 saturating_sub 防止下溢
        let mut bytes_needed = (bits_needed + 7) / 8; // 这是一种常见的 向上取整除法技巧（Ceiling Division Trick），用于计算容纳指定位数所需的最小字节数（当`bits_needed`不是8的倍数时，加上7就会使得总和至少达到下一个8的倍数，从而在除以8时得到正确地向上取整的结果）
        let max_bytes_needed = (64 - self.bits_in_buffer) / 8;
        if bytes_needed > max_bytes_needed {
            bytes_needed = max_bytes_needed;
        }
        if bytes_needed > 0 {
            let mut buf = [0u8; 8]; // 注意这里没有用 vector（堆上分配） 而是使用了栈上分配数组，这是一个性能优化
            let slice = &mut buf[..bytes_needed];
            if self.inner.read(slice)? < bytes_needed {
                return Err(BitReadWriteError::UnexpectedEof.into());
            };
            for &mut b in slice {
                // 所谓低地址就是如果顺序的将一块字流读取出来，首个字节索引是 0，第二个字节索引是 1，以此类推，0 就是低地址，也就是最读到的（索引最大的那个）必然是高地址
                // 大端序时来的数据越晚，左移的位数就越少，这样最后一个数据（最高地址数据）就在最右边（最低位）
                // 小端序时来的数据越晚，左移的位数就越多，这样最后一个数据（最高地址数据）就在最左边（最高位）
                let shift = match self.byte_order {
                    ByteOrder::BigEndian => {
                        // 大端序的低位字节存储在高地址，高位字节存储在低地址
                        // 大端序读取时，新读到数据（高地址数据）总是放置在比特缓冲区剩余空间的最低位（最右边）
                        let s = 64u32 - 8u32 - self.bits_in_buffer as u32; // shift = 64 - 8 - available_bits
                        s
                    }
                    ByteOrder::LittleEndian => {
                        // 小端序的低位字节存储在低地址，高位字节存储在高地址
                        // 小端序读取时，新读到数据（高地址数据）总是要放置在比特缓冲区的最高位（最左边）
                        let s = self.bits_in_buffer as u32;
                        s
                    }
                };
                // 将新读到数据（高地址数据）左移 shift 位，然后与比特缓冲区进行或运算，这样就是将新数据放到了比特缓冲区的最高位（最左边）
                self.bits_buffer |= u64::from(b).wrapping_shl(shift);
                // 更新比特缓冲区可用比特数
                self.bits_in_buffer = (self.bits_in_buffer + 8).min(64);
            }
        }
        Ok(())
    }

    fn get_from_bits_buffer(&mut self, n: usize, take: bool) -> std::io::Result<u64> {
        let bit_value = match self.byte_order {
            ByteOrder::BigEndian => {
                // 提取比特缓冲区高位 n 位（从左数的 n 位）
                let value = self.bits_buffer >> (64 - n);
                value
            }
            ByteOrder::LittleEndian => {
                // 用位掩码提取低 n 位
                let mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
                let value = self.bits_buffer & mask;
                value
            }
        };
        if take {
            if n == 64 {
                self.bits_buffer = 0;
            } else {
                match self.byte_order {
                    ByteOrder::BigEndian => {
                        self.bits_buffer <<= n;
                    }
                    ByteOrder::LittleEndian => {
                        self.bits_buffer >>= n;
                    }
                }
            }

            self.bits_in_buffer -= n;
        }
        Ok(bit_value)
    }
}

impl<R: Read> BitReader<R> {
    /// Returns `true` if at byte boundary (no pending bits)
    ///
    /// When true:
    /// - `read()` operations are permitted
    /// - Next bit read will start from a fresh byte
    pub fn is_byte_aligned(&self) -> bool {
        self.bits_in_buffer % 8 == 0
    }
}

impl<R: Read> BitRead for BitReader<R> {
    type Output = u64;

    /// Reads exactly `n` bits from the stream (1-64 bits)
    ///
    /// # Arguments
    /// * `n` - Number of bits to read (1 to 64 inclusive)
    ///
    /// # Returns
    /// Bits read
    ///
    /// # Errors
    /// Returns error if `n` is not between 1-64 or not enough bits are available
    fn read_bits(&mut self, n: usize) -> std::io::Result<Self::Output> {
        // 校验 n
        if n == 0 || n > 64 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }

        // 填充比特缓冲区
        self.put_into_bits_buffer(n)?;

        // 从比特缓冲区取 n 比特，并且消费掉
        self.get_from_bits_buffer(n, true)
    }
}

impl<R: Read> Read for BitReader<R> {
    /// Reads bytes from the underlying bit stream.
    ///
    /// This method behaves differently depending on the bit buffer state:
    /// - When the bit buffer is **empty** (byte-aligned state), it delegates directly to the inner reader
    /// - When the bit buffer contains **unconsumed bits** (non-byte-aligned state), it returns an
    ///   [`UnalignedAccess`](BitReadWriteError::UnalignedAccess) error
    ///
    /// # Byte Alignment Requirement
    /// The fundamental reason for this behavior is **bit stream integrity**. When partially consumed
    /// bits exist in the buffer:
    /// 1. Direct byte access would bypass the bit buffer's state tracking
    /// 2. Reading bytes would consume underlying bytes that contain the *remaining portions* of
    ///    partially read bit sequences
    /// 3. This would irreversibly corrupt the bit-level parsing state
    ///
    /// # Correct Usage
    /// To read byte data:
    /// 1. Use `is_byte_aligned()` to check if you're in byte-aligned state
    /// 2. For mixed bit/byte reading, always consume all bits in the current byte before reading bytes
    ///
    /// # Errors
    /// Returns `BitReadWriteError::UnalignedAccess` (wrapped in `io::Error`) when called with
    /// non-empty bit buffer. This prevents:
    /// - Undefined state transitions
    /// - Silent data corruption
    /// - Loss of partially buffered bits
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut written = 0;

        // 1) 如果完全空，直接读取
        if self.bits_in_buffer == 0 {
            return self.inner.read(buf);
        }

        // 2) 如果有残留，但已经是整字节边界（8 的倍数），先拆 buffer
        if self.bits_in_buffer % 8 == 0 {
            // 一直拆，直到 buffer 中不再有整字节或 buf 写满
            while self.bits_in_buffer >= 8 && written < buf.len() {
                // 每次取 8 位并消费
                let byte = self.get_from_bits_buffer(8, true)? as u8;
                buf[written] = byte;
                written += 1;
            }

            // 拆完后，buffer 要么空，要么剩 <8 位（此处一定是空，因为 bits_in_buffer%8==0）
            // 剩余 buf 空间，再走一次底层读以获取后续字节
            if written < buf.len() {
                let n = self.inner.read(&mut buf[written..])?;
                written += n;
            }

            return Ok(written);
        }

        // 3) 剩余 bits 不是 8 的倍数 —— 非字节对齐，禁止直接读
        Err(BitReadWriteError::UnalignedAccess.into())
    }
}

// ------------------------------- PeekableBitReader ------------------------------- //

pub struct PeekableBitReader<R: Read> {
    inner: BitReader<R>,
}

impl<R: Read> PeekableBitReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: BitReader::new(inner),
        }
    }

    pub fn with_byte_order(inner: R) -> Self {
        Self {
            inner: BitReader::with_byte_order(ByteOrder::LittleEndian, inner),
        }
    }
}

impl<R: Read> BitRead for PeekableBitReader<R> {
    type Output = u64;

    fn read_bits(&mut self, n: usize) -> std::io::Result<Self::Output> {
        self.inner.read_bits(n)
    }
}

impl<R: Read> BitPeek for PeekableBitReader<R> {
    type Output = u64;

    fn peek_bits(&mut self, n: usize) -> std::io::Result<Self::Output> {
        if n == 0 || n > 64 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }

        // 填充比特缓冲区
        self.inner.put_into_bits_buffer(n)?;

        // 从比特缓冲区取 n 比特，但是并不消费掉
        self.inner.get_from_bits_buffer(n, false)
    }
}

// ------------------------------- BulkBitReader ------------------------------- //

pub struct BulkBitReader<R: Read> {
    inner: BitReader<R>,
}

impl<R: Read> BulkBitReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: BitReader::new(inner),
        }
    }

    pub fn with_endianness(endianness: ByteOrder, inner: R) -> Self {
        Self {
            inner: BitReader::with_byte_order(endianness, inner),
        }
    }
}

impl<R: Read> BitRead for BulkBitReader<R> {
    type Output = Vec<u64>;

    fn read_bits(&mut self, n: usize) -> std::io::Result<Self::Output> {
        if n == 0 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }
        let mut remaining = n;
        let mut chunks = Vec::with_capacity((n + 63) / 64);
        while remaining > 0 {
            let take = remaining.min(64);
            chunks.push(self.inner.read_bits(take)?);
            remaining -= take;
        }
        Ok(chunks)
    }
}
