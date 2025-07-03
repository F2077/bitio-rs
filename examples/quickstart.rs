use bitio_rs::byte_order::ByteOrder;
use bitio_rs::reader::BitReader;
use bitio_rs::traits::{BitRead, BitWrite};
use bitio_rs::writer::BitWriter;
use itertools::Itertools;
use std::io::{Cursor, Read, Result, Write};

fn main() -> Result<()> {
    // ===== Writing Demonstrations =====
    println!("--- Writing Demonstrations ---");

    // Demo 1: 3-bit + 16-bit write
    let mut buffer1 = Vec::new();
    {
        let mut bit_writer = BitWriter::new(Cursor::new(&mut buffer1));
        bit_writer.write_bits(0b101, 3)?;
        bit_writer.write_bits(0b1010101111001101, 16)?; // 0xABCD
        bit_writer.flush()?;
    }
    println!(
        "Demo1: {}",
        buffer1
            .iter()
            .format_with(" ", |b, f| f(&format_args!("{:08b}", b)))
    );

    // Demo 2: Mixed byte/bit write
    let mut buffer2 = Vec::new();
    {
        let mut bit_writer = BitWriter::new(Cursor::new(&mut buffer2));
        bit_writer.write(&[0b00010001, 0b00100010])?; // 0x11, 0x22
        bit_writer.write_bits(0b11011, 5)?;
        bit_writer.write(&[0b00110011])?; // 0x33
    }
    println!(
        "\nDemo2: {}",
        buffer2
            .iter()
            .format_with(" ", |b, f| f(&format_args!("{:08b}", b)))
    );

    // ===== Reading Demonstrations =====
    println!("\n--- Reading Demonstrations ---");

    // Independent test data
    let read_data = vec![0b10101010, 0b10111011, 0b11001100];

    // Demo 3: Bit reading
    let mut bit_reader = BitReader::with_byte_order(ByteOrder::BigEndian, Cursor::new(&read_data));

    let bits3 = bit_reader.read_bits(3)?; // 0b101
    let bits8 = bit_reader.read_bits(8)?; // 0b01010111
    println!("Read 3+8 bits: {:03b} {:08b}", bits3, bits8);

    // Demo 4: Byte reading
    let mut buf = [0u8; 2];
    let bytes_read = bit_reader.read(&mut buf)?;
    println!(
        "Subsequent bytes: {}",
        &buf[..bytes_read]
            .iter()
            .map(|b| format!("{:08b}", b))
            .collect::<Vec<String>>()
            .join(" ")
    );

    // Demo 5: Misalignment error
    let mut error_reader = BitReader::new(Cursor::new(&[0b10101010]));
    error_reader.read_bits(1)?;
    match error_reader.read(&mut [0u8; 1]) {
        Ok(_) => println!("Error: Misaligned read succeeded unexpectedly"),
        Err(e) => println!("Expected error: {}", e),
    }

    Ok(())
}
