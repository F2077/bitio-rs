use crate::byte_order::ByteOrder;
use crate::error::BitReadWriteError;
use crate::traits::BitWrite;
use std::fmt::Debug;
use std::io::{BufWriter, Result, Write};

pub struct BitWriter<W: Write> {
    byte_order: ByteOrder,
    inner: BufWriter<W>, // 用 BufWriter<W> 能避免频繁的系统调用

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
            inner: BufWriter::new(inner),
            bits_buffer: 0,
            bits_in_buffer: 0,
        }
    }
}

impl<W: Write> BitWriter<W> {
    /// 将对齐的（完整的）字节写入底层的写入器
    fn write_aligned_bytes_to_inner(&mut self) -> Result<()> {
        // 先算出有多少对齐的字节待写入底层
        // 注意本操作只会处理对齐的字节
        let count = self.bits_in_buffer / 8;
        if count == 0 {
            return Ok(());
        }

        let mut buf = Vec::with_capacity(count);
        for _ in 0..count {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> 56) as u8, // 大端序每次都从比特缓冲区左边取 8 位，也就是 1 字节，注意这里没有改变比特缓冲区本身
                ByteOrder::LittleEndian => self.bits_buffer as u8, // 小端序每次都从比特缓冲区右边取 8 位，也就是 1 字节，注意这里没有改变比特缓冲区本身
            };
            buf.push(byte);

            match self.byte_order {
                ByteOrder::BigEndian => {
                    self.bits_buffer <<= 8; // 大端序每次从左边取完比特缓冲区 1 字节后，要从左边消除掉已经取出的 8 位，注意这里改变了比特缓冲区本身
                }
                ByteOrder::LittleEndian => {
                    self.bits_buffer >>= 8; // 小端序每次从右边取完比特缓冲区 1 字节后，要从右边消除掉已经取出的 8 位，注意这里改变了比特缓冲区本身
                }
            }
            self.bits_in_buffer -= 8; // 更改比特缓冲区位计数
        }
        self.inner.write_all(&buf)?; // 一次写多个字节能减少潜在的系统调用
        Ok(())
    }

    /// 将比特缓冲区尾部的不足 1 字节的数据写入底层的写入器，注意，这个函数只能在比特缓冲区中剩余位不足 1 字节（8 比特）时调用才有意义
    fn write_residual_partial_byte_to_inner(&mut self) -> Result<()> {
        if self.bits_in_buffer > 0 && self.bits_in_buffer < 8 {
            let byte = match self.byte_order {
                ByteOrder::BigEndian => (self.bits_buffer >> 56) as u8, // 对于大端序，将比特缓冲区最左边剩余的不足 1 字节的位写入底层的写入器
                ByteOrder::LittleEndian => self.bits_buffer as u8, // 对于小端序，将比特缓冲区最右边剩余的不足 1 字节的位写入底层的写入器
            };
            self.inner.write_all(&[byte])?;
            self.bits_buffer = 0; // 清零比特缓冲区
            self.bits_in_buffer = 0; // 清零比特缓冲区计数
        }
        Ok(())
    }
}

impl<W: Write> Write for BitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // 在写入新来的字节组到底层写入器之前，先确保比特缓冲区中对齐的字节被写入底层的写入器
        self.write_aligned_bytes_to_inner()?;

        if self.bits_in_buffer == 0 {
            // 如果执行完将比特缓冲区中所有对齐字节都写入底层的写入器后，如果比特缓冲区已经清零（此时已是干净的状态），那么就可以将新来的字节组直接写入底层的写入器（高速）
            self.inner.write(buf)?;
            return Ok(buf.len());
        }

        // 如果执行完将比特缓冲区中所有对齐字节都写入底层的写入器后，比特缓冲区中还有剩余的位（也就是未对齐为 1 字节的位，比如 3 比特），那么就需要将字节组的每个字节都执行 “比特写”（在这个过程中实际上是先将所有自己组的字节都写到比特缓冲区然后由后续逻辑从比特缓冲区写到底层写入器，也就是不允许绕过比特缓冲区） 这样才能保证底层写入器是无空隙的（这样速度较字节组直写要慢，但是我们的底层写入器保证是 BufWriter 因此不会慢太多）
        for &b in buf {
            self.write_bits(b as u64, 8)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        // 注意冲刷操作一定要把比特缓冲区的残尾字节写入底层写入器，否则底层写入器就少尾部数据了
        self.write_residual_partial_byte_to_inner()?;
        self.inner.flush()
    }
}

impl<W: Write> Drop for BitWriter<W> {
    fn drop(&mut self) {
        // 注意这里忽略了错误因为 Drop 里无法 panic
        let _ = self.write_residual_partial_byte_to_inner();
        let _ = self.inner.flush();
    }
}

impl<W: Write> BitWrite for BitWriter<W> {
    fn write_bits(&mut self, value: u64, n: usize) -> Result<()> {
        // 校验 n
        if n == 0 || n > 64 {
            return Err(BitReadWriteError::InvalidBitCount(n).into());
        }

        let mut remaining = n;
        let mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 }; // (1u64 << n) - 1 就是低位连续 n 个 1，高位全是 0
        let mut val = value & mask; // 用掩码取出 n 位有效位，无效的位被丢弃

        while remaining > 0 {
            let available = 64 - self.bits_in_buffer;
            let to_insert = remaining.min(available);
            let insert_at_next_round = remaining - to_insert;
            let to_insert_val = val >> insert_at_next_round; // 注意这里没有改变 val 本身，而是用 val 的一部分建立了新值

            match self.byte_order {
                ByteOrder::BigEndian => {
                    self.bits_buffer |= to_insert_val << (available - to_insert); // 大端序时是把值从比特缓冲区的左边往右边堆（可以想象比特缓冲区是一个能容纳 64 块砖的长条盒子，大端序就是来一块砖就从左开始码放）
                }
                ByteOrder::LittleEndian => {
                    self.bits_buffer |= to_insert_val << self.bits_in_buffer; // 小端序时是把值从比特缓冲区的右边往左边堆（可以想象比特缓冲区是一个能容纳 64 块砖的长条盒子，小端序就是来一块砖就从右开始码放）
                }
            }

            self.bits_in_buffer += to_insert; // 更新比特缓冲区中已有的位数
            remaining -= to_insert; // 更新剩余的要插入的位数

            if insert_at_next_round > 0 {
                val &= (1u64 << insert_at_next_round) - 1; //  (1u64 << insert_at_next_round) - 1 又是一个掩码，用下一轮要插入的位数来更新 val，相当于丢弃了 val 中本轮已经插入过的位，注意这里是直接修改了 val 本身
            }

            // 每凑够（包括大于的情况）1 字节就触发一次写入底层写入器的操作
            if self.bits_in_buffer >= 8 || remaining == 0 {
                self.write_aligned_bytes_to_inner()?; // 注意只能将对其的部分写入底层写入器，如果将未对齐的也写入了，后续再有新的字节组过来时，底层写入器就会因为本次写入了部分字节后出现位的断档
            }
        }

        Ok(())
    }
}
