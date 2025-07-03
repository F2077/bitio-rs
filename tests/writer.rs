#[cfg(test)]
mod tests {
    use bitio_rs::byte_order::ByteOrder;
    use bitio_rs::traits::BitWrite;
    use bitio_rs::writer::BitWriter;
    use std::io::{Cursor, Write};

    #[test]
    fn test_write_bits_big_endian() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buf);
        writer.write_bits(0b1010, 4).unwrap();
        writer.write_bits(0b1100, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buf.into_inner(), vec![0xAC]);
    }

    #[test]
    fn test_write_bits_little_endian() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buf);
        writer.write_bits(0b1010, 4).unwrap();
        writer.write_bits(0b1100, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buf.into_inner(), vec![0xCA]);
    }

    #[test]
    fn test_mixed_write_and_write_bits() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::new(&mut buf);
        writer.write_bits(0xFF, 8).unwrap();
        let data = [0x11, 0x22, 0x33];
        writer.write(&data).unwrap();
        writer.write_bits(0b101, 3).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buf.into_inner(), vec![0xFF, 0x11, 0x22, 0x33, 0xA0]);
    }

    #[test]
    fn test_write_bytes_when_buffer_empty() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::new(&mut buf);
        let data = [1, 2, 3, 4];
        let n = writer.write(&data).unwrap();
        writer.flush().unwrap();
        assert_eq!(n, 4);
        drop(writer);
        assert_eq!(buf.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_partial_byte_flush_padding() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::new(&mut buf);
        writer.write_bits(0b111, 3).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buf.into_inner(), vec![0xE0]);
    }

    #[test]
    fn test_mixed_write_and_write_bits_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write(&[0xFF]).unwrap();
        writer.write_bits(0x11, 8).unwrap();
        writer.write_bits(0x22, 8).unwrap();
        writer.write_bits(0x33, 8).unwrap();
        writer.write_bits(0x14, 5).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xFF, 0x11, 0x22, 0x33, 0xA0]);
    }

    #[test]
    fn test_mixed_write_and_write_bits_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write(&[0xFF]).unwrap();
        writer.write_bits(0x11, 8).unwrap();
        writer.write_bits(0x22, 8).unwrap();
        writer.write_bits(0x33, 8).unwrap();
        writer.write_bits(0x14, 5).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xFF, 0x11, 0x22, 0x33, 0x14]);
    }

    #[test]
    fn test_write_zero_bits() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::new(&mut buffer);
        assert!(writer.write_bits(0x1234, 0).is_err());
    }

    #[test]
    fn test_write_64_bits_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write_bits(0x0123456789ABCDEF, 64).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn test_write_64_bits_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write_bits(0x0123456789ABCDEF, 64).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]);
    }

    #[test]
    fn test_write_65_bits() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::new(&mut buffer);
        assert!(writer.write_bits(0x1, 65).is_err());
    }

    #[test]
    fn test_7_bits_then_1_bit_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write_bits(0x7F, 7).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xFF]);
    }

    #[test]
    fn test_7_bits_then_1_bit_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write_bits(0x7F, 7).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xFF]);
    }

    #[test]
    fn test_byte_then_7_bits_then_1_bit_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write(&[0xAA]).unwrap();
        writer.write_bits(0x7F, 7).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xAA, 0xFF]);
    }

    #[test]
    fn test_byte_then_7_bits_then_1_bit_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write(&[0xAA]).unwrap();
        writer.write_bits(0x7F, 7).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xAA, 0xFF]);
    }

    #[test]
    fn test_flush_in_middle_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        writer.write(&[0xAA]).unwrap();
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xF0, 0xAA, 0xF0]);
    }

    #[test]
    fn test_flush_in_middle_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        writer.write(&[0xAA]).unwrap();
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0x0F, 0xAA, 0x0F]);
    }

    #[test]
    fn test_write_64_bits_then_1_bit_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write_bits(0x0123456789ABCDEF, 64).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(
            buffer,
            vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x80]
        );
    }

    #[test]
    fn test_write_64_bits_then_1_bit_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write_bits(0x0123456789ABCDEF, 64).unwrap();
        writer.write_bits(0x1, 1).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(
            buffer,
            vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, 0x01]
        );
    }

    #[test]
    fn test_partial_byte_flush_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::BigEndian, &mut buffer);
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0xF0]);
    }

    #[test]
    fn test_partial_byte_flush_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buffer);
        writer.write_bits(0x0F, 4).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0x0F]);
    }
}
