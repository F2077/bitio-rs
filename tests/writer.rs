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
        // write 4 bits: 0b1010, then 4 bits: 0b1100 => should form byte: 0b10101100 == 0xAC
        writer.write_bits(0b1010, 4).unwrap();
        writer.write_bits(0b1100, 4).unwrap();
        writer.flush().unwrap();
        assert_eq!(buf.into_inner(), vec![0xAC]);
    }

    #[test]
    fn test_write_bits_little_endian() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::with_byte_order(ByteOrder::LittleEndian, &mut buf);
        // little endian: write 4 bits: 0b1010 (lowest bits), then 4 bits: 0b1100 => byte: 0b11001010 == 0xCA
        writer.write_bits(0b1010, 4).unwrap();
        writer.write_bits(0b1100, 4).unwrap();
        writer.flush().unwrap();
        assert_eq!(buf.into_inner(), vec![0xCA]);
    }

    #[test]
    fn test_mixed_write_and_write_bits() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::new(&mut buf);
        writer.write_bits(0xFF, 8).unwrap(); // full byte
        let data = [0x11, 0x22, 0x33];
        writer.write(&data).unwrap();
        writer.write_bits(0b101, 3).unwrap();
        writer.flush().unwrap();
        // Expect: 0xFF, 0x11, 0x22, 0x33, then a byte with top 3 bits 10100000 == 0xA0
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
        assert_eq!(buf.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_partial_byte_flush_padding() {
        let mut buf = Cursor::new(Vec::new());
        let mut writer = BitWriter::new(&mut buf);
        // write 3 bits: 0b111 => padded in flush to 11100000 == 0xE0
        writer.write_bits(0b111, 3).unwrap();
        writer.flush().unwrap();
        assert_eq!(buf.into_inner(), vec![0xE0]);
    }
}
