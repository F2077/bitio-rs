use crate::byte_order::ByteOrder;
use crate::traits::BitWrite;
use std::io::{Error, ErrorKind, Result, Write};

pub struct BitWriter<W: Write> {
    byte_order: ByteOrder,
    inner: W,

    bits_buffer: u64,
    bits_in_buffer: usize,
}

impl<W: Write> BitWriter<W> {
    pub fn new(inner: W) -> Self {
        Self::with_byte_order(ByteOrder::BigEndian, inner)
    }

    pub fn with_byte_order(byte_order: ByteOrder, inner: W) -> Self {
        Self {
            byte_order,
            inner,
            bits_buffer: 0,
            bits_in_buffer: 0,
        }
    }

    /// 刷新缓冲区中完整的字节（8位的整数倍）
    fn flush_complete_bytes(&mut self) -> Result<()> {
        while self.bits_in_buffer >= 8 {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> 56) as u8,
                ByteOrder::LittleEndian => self.bits_buffer as u8,
            };
            self.inner.write_all(&[byte])?;

            match self.byte_order {
                ByteOrder::BigEndian => {
                    // 移除最高的字节
                    self.bits_buffer <<= 8;
                    self.bits_in_buffer -= 8;
                }
                ByteOrder::LittleEndian => {
                    self.bits_buffer >>= 8;
                    self.bits_in_buffer -= 8;
                }
            }
        }
        Ok(())
    }

    /// 强制刷新所有位（包括不足8位的部分）
    fn flush_partial_byte(&mut self) -> Result<()> {
        if self.bits_in_buffer > 0 {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> 56) as u8,
                ByteOrder::LittleEndian => self.bits_buffer as u8,
            };
            self.inner.write_all(&[byte])?;
            self.bits_buffer = 0;
            self.bits_in_buffer = 0;
        }
        Ok(())
    }
}

impl<W: Write> Write for BitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.flush_complete_bytes()?;

        if self.bits_in_buffer == 0 {
            self.inner.write(buf)
        } else {
            let mut processed = 0;
            let mut k = self.bits_in_buffer;
            let free_bits = 64 - k;
            let bytes_to_process = (free_bits / 8).min(buf.len());

            for &byte in &buf[..bytes_to_process] {
                match self.byte_order {
                    ByteOrder::BigEndian => {
                        self.bits_buffer |= (byte as u64) << (56 - k);
                    }
                    ByteOrder::LittleEndian => {
                        self.bits_buffer |= (byte as u64) << k;
                    }
                }
                processed += 1;
                k += 8;
            }
            self.bits_in_buffer += 8 * processed;

            self.flush_complete_bytes()?;

            if processed < buf.len() {
                self.inner.write_all(&buf[processed..])?;
                processed = buf.len();
            }

            Ok(processed)
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.flush_partial_byte()?;
        self.inner.flush()
    }
}

impl<W: Write> BitWrite for BitWriter<W> {
    fn write_bits(&mut self, value: u64, n: usize) -> Result<()> {
        // 限制 n 的范围在 1..=64
        if n == 0 {
            return Ok(());
        }
        if n > 64 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Cannot write {} bits, maximum is 64", n),
            ));
        }

        let mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
        let mut remaining = n;
        let mut val = value & mask;

        while remaining > 0 {
            let available = 64 - self.bits_in_buffer;
            let to_insert = remaining.min(available);
            let shift = remaining - to_insert;
            let to_insert_val = val >> shift;

            match self.byte_order {
                ByteOrder::BigEndian => {
                    self.bits_buffer |= to_insert_val << (available - to_insert);
                }
                ByteOrder::LittleEndian => {
                    self.bits_buffer |= to_insert_val << self.bits_in_buffer;
                }
            }

            self.bits_in_buffer += to_insert;
            remaining -= to_insert;
            if shift > 0 {
                val &= (1u64 << shift) - 1;
            }

            if self.bits_in_buffer >= 8 || remaining == 0 {
                self.flush_complete_bytes()?;
            }
        }

        Ok(())
    }
}
