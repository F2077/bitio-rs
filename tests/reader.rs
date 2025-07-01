#[cfg(test)]
mod tests {
    use bitio_rs::byte_order::ByteOrder;
    use bitio_rs::reader::{BitReader, BulkBitReader};
    use bitio_rs::traits::{BitPeek, BitRead};
    use std::io::Cursor;
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

    #[test]
    fn read_bool_sequence() {
        let data = [0b1010_0001u8];
        let mut reader = BitReader::new(Cursor::new(data));
        // read 3 bools
        let b1 = reader.read_bool().unwrap();
        let b2 = reader.read_bool().unwrap();
        let b3 = reader.read_bool().unwrap();
        assert_eq!((b1, b2, b3), (true, false, true));
    }

    #[test]
    fn peek_bool_and_read_bool_consistency() {
        let data = [0b1101_1000u8];
        let mut reader = BitReader::new(Cursor::new(data));
        // first two bits are 1,1
        assert_eq!(reader.peek_bool().unwrap(), true);
        assert_eq!(reader.read_bool().unwrap(), true);
        assert_eq!(reader.peek_bool().unwrap(), true);
        assert_eq!(reader.read_bool().unwrap(), true);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.read_bool().unwrap(), false);
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
    fn bulk_read_bool_sequence() {
        let data = vec![0b1010_0001u8; 2];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        // read 3 bools
        let b1 = reader.read_bool().unwrap();
        let b2 = reader.read_bool().unwrap();
        let b3 = reader.read_bool().unwrap();
        assert_eq!((b1, b2, b3), (true, false, true));
    }

    #[test]
    fn bulk_peek_bits_does_not_consume() {
        let data = vec![0b1111_0000u8; 2];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        let first = reader.peek_bits(12).unwrap();
        let second = reader.read_bits(12).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn bulk_peek_bool_and_read_bool_consistency() {
        let data = vec![0b1101_1000u8];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        // first two bits are 1,1
        assert_eq!(reader.peek_bool().unwrap(), true);
        assert_eq!(reader.read_bool().unwrap(), true);
        assert_eq!(reader.peek_bool().unwrap(), true);
        assert_eq!(reader.read_bool().unwrap(), true);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.peek_bool().unwrap(), false);
        assert_eq!(reader.read_bool().unwrap(), false);
    }

    #[test]
    fn bulk_read_bool_unexpected_eof() {
        let data = vec![];
        let mut reader = BulkBitReader::new(Cursor::new(data));
        assert!(reader.read_bool().is_err());
    }
}
