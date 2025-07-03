#[cfg(test)]
mod tests {
    use bitio_rs::byte_order::ByteOrder;
    use bitio_rs::reader::{BitReader, BulkBitReader, PeekableBitReader};
    use bitio_rs::traits::{BitPeek, BitRead};
    use std::io::{Cursor, ErrorKind, Read};
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
        let mut reader = PeekableBitReader::new(Cursor::new(data));
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
    fn test_byte_aligned_read() {
        // 测试在没有任何位读取的情况下，直接读取字节
        let data = vec![0xAA, 0xBB, 0xCC];
        let mut reader = BitReader::new(Cursor::new(data.clone()));
        let mut buf = [0u8; 3];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(buf, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_read_bits_then_error_on_unaligned_read() {
        // 先读 4 位，剩余 4 位未对齐，read 应该返回错误
        let data = vec![0b1010_1111, 0x11];
        let mut reader = BitReader::new(Cursor::new(data));
        // 读高 4 位 => 0b1010
        let bits = reader.read_bits(4).unwrap();
        assert_eq!(bits, 0b1010);

        // 此时 bits_in_buffer = 4，非整字节对齐
        let mut buf = [0u8; 1];
        let err = reader.read(&mut buf).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other); // 包装后的 UnalignedAccess
    }

    #[test]
    fn test_partial_buffer_consumption() {
        // 如果内部缓冲中有整字节，也应先拆出来
        let data = vec![0xAB, 0xCD];

        // 这里我们先借助 read_bits 读满 8 位，留在 buffer
        let mut reader = BitReader::new(Cursor::new(data.clone()));
        let b = reader.read_bits(8).unwrap() as u8;
        assert_eq!(b, 0xAB);

        // 此时 bits_in_buffer == 0，因为刚好读完一个字节
        // 再 read_bytes 应直接从 inner 读剩余
        let mut buf = [0u8; 1];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 1);
        assert_eq!(buf, [0xCD]);
    }

    #[test]
    fn test_read_bytes_auto_align_and_consume_buffer() {
        // 测试在缓冲区有完整字节时，read 会先拆 buffer 中的字节
        let data = vec![0xFE, 0xEF, 0x01];
        let mut reader = BitReader::new(Cursor::new(data));

        // 先读 16 位，这会把 0xFE,0xEF 都装入 buffer
        let chunks = reader.read_bits(16).unwrap();
        assert_eq!(chunks, 0xFEEF);

        // buffer 中 bits_in_buffer == 0 (已经消费完)
        // 读剩余字节
        let mut buf = [0u8; 1];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 1);
        assert_eq!(buf, [0x01]);
    }
}
