#[cfg(test)]
mod tests {
    use bitio_rs::byte_order::ByteOrder;
    use bitio_rs::reader::{BitReader, BulkBitReader};
    use bitio_rs::traits::{BitPeek, BitRead};
    use std::io::{Cursor, Read};
    // ------------------------------- BitReader tests ------------------------------- //

    #[test]
    fn test_big_endian_read_bit() {
        let data = [0b1010_1010];
        let mut reader = BitReader::new(Cursor::new(data));
        assert_eq!(reader.read_bits(1).unwrap(), 1); // 最高位
        assert_eq!(reader.read_bits(1).unwrap(), 0);
        assert_eq!(reader.read_bits(1).unwrap(), 1);
        assert_eq!(reader.read_bits(5).unwrap(), 0b01010); // 剩余5位
    }

    #[test]
    fn test_little_endian_read_bit() {
        let data = [0b1010_1010];
        let mut reader = BitReader::with_byte_order(ByteOrder::LittleEndian, Cursor::new(data));
        assert_eq!(reader.read_bits(1).unwrap(), 0); // 最低位 (位0)
        assert_eq!(reader.read_bits(1).unwrap(), 1);
        assert_eq!(reader.read_bits(1).unwrap(), 0);
        assert_eq!(reader.read_bits(5).unwrap(), 0b10101); // 剩余5位 (位3-7)
    }

    #[test]
    fn test_big_endian_cross_byte_read() {
        let data = [0b1100_1100, 0b1010_1010];
        let mut reader = BitReader::new(Cursor::new(data));
        assert_eq!(reader.read_bits(3).unwrap(), 0b110); // 第一个字节的高3位
        assert_eq!(reader.read_bits(10).unwrap(), 0b0_11001010_1); // 剩余5位 + 第二个字节的8位（高位在前）
    }

    #[test]
    fn test_little_endian_cross_byte_read() {
        let data = [0b0000_0001, 0b1000_0000]; // 字节0: 0x01, 字节1: 0x80
        let mut reader = BitReader::with_byte_order(ByteOrder::LittleEndian, Cursor::new(data));
        assert_eq!(reader.read_bits(1).unwrap(), 1); // 字节0的位0
        assert_eq!(reader.read_bits(8).unwrap(), 0); // 字节0的剩余7位（全0） + 字节1的位0（0）
        assert_eq!(reader.read_bits(1).unwrap(), 0); // 字节1的位1
        assert_eq!(reader.read_bits(1).unwrap(), 0); // 字节1的位2
        assert_eq!(reader.read_bits(5).unwrap(), 0b10000); // 字节1的剩余5位（位3-7）
    }

    #[test]
    fn test_big_endian_cross_bytes_read() {
        let data = [
            0b0000_0001,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1100_1100,
            0b1100_1100,
            0b0000_0001,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1100_1100,
            0b1100_1100,
            0b0000_0001,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1100_1100,
            0b1100_1100,
        ]; // 字节0: 0x01, 字节1: 0x80
        let mut reader = BitReader::with_byte_order(ByteOrder::BigEndian, Cursor::new(data));
        assert_eq!(
            reader.read_bits(64).unwrap(),
            0b00000001_10000000_10000000_10000000_10000000_10000000_11001100_11001100
        ); // 字节0的位0
        assert_eq!(reader.read_bits(8).unwrap(), 0b0000_0001);
        assert_eq!(reader.read_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_bits(1).unwrap(), 0b0);
        assert_eq!(reader.read_bits(1).unwrap(), 0b0);
    }

    #[test]
    fn test_peek_bits() {
        let data = [0b1100_1111];
        let mut reader = BitReader::new(Cursor::new(data));
        assert_eq!(reader.peek_bits(4).unwrap(), 0b1100); // 查看前4位
        assert_eq!(reader.read_bits(4).unwrap(), 0b1100); // 实际读取（应相同）
        assert_eq!(reader.peek_bits(4).unwrap(), 0b1111); // 查看接下来的4位
    }

    #[test]
    fn test_aligned_byte_read_big_endian() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = BitReader::new(Cursor::new(data));
        assert_eq!(reader.read_bits(8).unwrap(), 0x12); // 直接读取整个字节
        assert_eq!(reader.read_bits(16).unwrap(), 0x3456); // 直接读取两个字节
    }

    #[test]
    fn test_aligned_byte_read_little_endian() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = BitReader::with_byte_order(ByteOrder::LittleEndian, Cursor::new(data));

        // 读取第一个字节
        assert_eq!(reader.read_bits(8).unwrap(), 0x12);

        // 读取接下来的两个字节（16位）
        // 小端序：先读0x34（低位），后读0x56（高位）
        assert_eq!(reader.read_bits(16).unwrap(), 0x5634);
    }

    #[test]
    fn test_little_endian_multi_byte() {
        let data = [0b0000_0001, 0b0000_0010, 0b0000_0011, 0b0000_0100]; // 0x01, 0x02, 0x03, 0x04
        let mut reader = BitReader::with_byte_order(ByteOrder::LittleEndian, Cursor::new(data));

        // 正确读取31位
        assert_eq!(reader.read_bits(31).unwrap(), 0x04030201);

        assert_eq!(reader.read_bits(1).unwrap(), 0b0);

        // 检查是否已到达文件末尾
        assert!(reader.read_bits(1).is_err());
    }

    #[test]
    fn test_read_past_end() {
        let data = [0x12, 0x34];
        let mut reader = BitReader::new(Cursor::new(data));

        // 读取16位（2字节）
        assert_eq!(reader.read_bits(16).unwrap(), 0x1234);

        // 尝试读取超过可用数据
        assert!(reader.read_bits(1).is_err());
        assert!(reader.read_bits(8).is_err());
    }

    #[test]
    fn test_partial_read_at_end() {
        let data = [0b1010_1010, 0b1100_1100];
        let mut reader = BitReader::new(Cursor::new(data));

        // 读取12位
        assert_eq!(reader.read_bits(12).unwrap(), 0b1010_1010_1100);

        // 尝试读取剩余4位（成功）
        assert_eq!(reader.read_bits(4).unwrap(), 0b1100);

        // 尝试读取更多（失败）
        assert!(reader.read_bits(1).is_err());
    }

    #[test]
    fn test_buffer_refill() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xAA]; // 9字节
        let mut reader = BitReader::new(Cursor::new(data));
        // 读取64位填满缓冲区
        assert_eq!(reader.read_bits(64).unwrap(), 0xFFFFFFFFFFFFFFFF);
        // 继续读取会触发重新填充
        assert_eq!(reader.read_bits(8).unwrap(), 0xAA);
    }

    // 测试读取0位时panic
    #[test]
    fn test_read_zero_bits_panics() {
        let data = [0xAA];
        let mut reader = BitReader::new(Cursor::new(data));
        assert!(reader.read_bits(0).is_err());
    }

    // ------------------------------- BulkBitReader tests ------------------------------- //

    #[test]
    fn bulk_read_bits_zero_error() {
        let data = vec![0u8; 1];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        assert!(reader.read_bits(0).is_err());
    }

    #[test]
    fn bulk_read_bits_across_chunks() {
        // 80 bits => two chunks: 64 + 16
        let data = vec![0xFFu8; 10];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        let chunks = reader.read_bits(80).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], u64::MAX);
        // next 16 bits all ones => lower 16 bits of next chunk
        assert_eq!(chunks[1] & 0xFFFF, 0xFFFF);
    }

    #[test]
    fn bulk_peek_bits_does_not_consume() {
        let data = vec![0b1111_0000u8; 2];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        let first = reader.peek_bits(12).unwrap();
        let second = reader.read_bits(12).unwrap();
        assert_eq!(first, second);
    }

    // --------------- Mixed byte/bit read tests --------------- //

    #[test]
    fn test_byte_then_bit() {
        let data = vec![0xAA, 0b10110011, 0xFF];
        let mut reader = BitReader::new(Cursor::new(data));
        // Read one full byte
        let mut byte = [0u8; 1];
        assert_eq!(reader.read(&mut byte).unwrap(), 1);
        assert_eq!(byte[0], 0xAA);
        // Now read 4 bits from next byte
        assert_eq!(reader.read_bits(4).unwrap(), 0b1011);
        // And then remaining 4 bits
        assert_eq!(reader.read_bits(4).unwrap(), 0b0011);
        // Then next aligned byte
        let mut b2 = [0u8; 1];
        assert_eq!(reader.read(&mut b2).unwrap(), 1);
        assert_eq!(b2[0], 0xFF);
    }

    #[test]
    fn test_bit_then_byte_then_bit() {
        let data = vec![0b11110000, 0xBB, 0b00001111];
        let mut reader = BitReader::new(Cursor::new(data));
        // Read 4 bits
        assert_eq!(reader.read_bits(4).unwrap(), 0b1111);
        // Align and read next byte
        let mut buf = [0u8; 1];
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], 0xBB);
        // Then read final 4 bits
        assert_eq!(reader.read_bits(4).unwrap(), 0b0000);
    }

    #[test]
    fn test_multiple_aligns_do_not_consume_extra() {
        let data = vec![0xCC, 0xDD];
        let mut reader = BitReader::new(Cursor::new(data));
        // Align twice consecutively
        let mut buf = [0u8; 2];
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        // Further align should read EOF (0 bytes)
        let mut buf2 = [0u8; 1];
        assert_eq!(reader.read(&mut buf2).unwrap(), 0);
    }
}
