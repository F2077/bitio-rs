use bitio_rs::byte_order::ByteOrder;
use bitio_rs::fast::reader::{FastBitReaderBig, FastBitReaderLittle};
use bitio_rs::reader::BitReader;
use bitio_rs::traits::BitRead;
use std::io::Cursor;

fn main() -> std::io::Result<()> {
    // === Standard BitReader (Big-Endian) ===
    let data_be = vec![0b1010_1100, 0b1111_0000];
    let mut reader = BitReader::with_byte_order(ByteOrder::BigEndian, Cursor::new(data_be.clone()));

    let first = reader.read_bits(3)?; // expect 0b101 = 5
    println!("Standard BE: first 3 bits = 0b{:03b} ({})", first, first);

    let second = reader.read_bits(4)?; // expect 0b0110 = 6
    println!("Standard BE: next 4 bits = 0b{:04b} ({})", second, second);

    // === FastBitReaderBig (Big-Endian, performance-critical) ===
    let data_fast = vec![0x12, 0x34, 0x56, 0x78];
    let mut fast_be = FastBitReaderBig::new(Cursor::new(data_fast.clone()));
    let value_be = fast_be.read_bits_fast(32)?; // expect 0x12345678
    println!("Fast BE: 32-bit value = 0x{:08X}", value_be);

    // === FastBitReaderLittle (Little-Endian, performance-critical) ===
    let mut fast_le = FastBitReaderLittle::new(Cursor::new(data_fast));
    let value_le = fast_le.read_bits_fast(32)?; // expect 0x78563412
    println!("Fast LE: 32-bit value = 0x{:08X}", value_le);

    Ok(())
}
