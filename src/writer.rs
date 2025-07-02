use crate::byte_order::ByteOrder;
use crate::traits::BitWrite;
use std::io::Write;

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
    fn flush_complete_bytes(&mut self) -> std::io::Result<()> {
        while self.bits_in_buffer >= 8 {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> (self.bits_in_buffer - 8)) as u8,
                ByteOrder::LittleEndian => self.bits_buffer as u8,
            };
            self.inner.write_all(&[byte])?;

            match self.byte_order {
                ByteOrder::BigEndian => {
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
    fn flush_partial_byte(&mut self) -> std::io::Result<()> {
        if self.bits_in_buffer > 0 {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> (self.bits_in_buffer - 8)) as u8,
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
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 先刷新缓冲区中的完整字节
        self.flush_complete_bytes()?;

        if self.bits_in_buffer == 0 {
            // 缓冲区为空时直接批量写入
            self.inner.write(buf)
        } else {
            // 批量处理字节：每次处理多个字节直到缓冲区接近满
            let mut processed = 0;
            let k = self.bits_in_buffer; // 当前缓冲区中的位数
            let free_bits = 64 - k; // 剩余可用位数

            // 计算能处理的完整字节数（至少处理1个字节）
            let bytes_to_process = (free_bits / 8).min(buf.len());

            // 批量处理字节
            for &byte in &buf[..bytes_to_process] {
                match self.byte_order {
                    ByteOrder::BigEndian => {
                        self.bits_buffer |= (byte as u64) << (free_bits - 8 * (processed + 1));
                    }
                    ByteOrder::LittleEndian => {
                        self.bits_buffer |= (byte as u64) << (k + 8 * processed);
                    }
                }
                processed += 1;
            }

            // 更新缓冲区计数
            self.bits_in_buffer += 8 * processed;

            // 刷新完整字节（可能产生多个完整字节）
            self.flush_complete_bytes()?;

            // 处理剩余字节
            if processed < buf.len() {
                // 剩余字节直接写入（因为缓冲区现在为空）
                self.inner.write_all(&buf[processed..])?;
                processed = buf.len();
            }

            Ok(processed)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_partial_byte()?;
        self.inner.flush()
    }
}

impl<W: Write> BitWrite for BitWriter<W> {
    fn write_bits(&mut self, value: u64, n: usize) -> std::io::Result<()> {
        let mask = if n == 64 { u64::MAX } else { (1 << n) - 1 };
        let actual_value = value & mask;
        let mut remaining = n;
        let mut val = actual_value;

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
            val &= (1 << shift) - 1; // 清除已处理的高位

            // 只在缓冲区满或需要刷新时才处理
            if self.bits_in_buffer == 64 || remaining == 0 {
                self.flush_complete_bytes()?;
            }
        }

        Ok(())
    }
}
