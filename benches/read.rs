use bitio_rs::byte_order::ByteOrder;
use bitio_rs::fast::reader::{FastBitReaderBig, FastBitReaderLittle};
use bitio_rs::reader::{BitReader, BulkBitReader};
use bitio_rs::traits::BitRead;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::io::Cursor;

fn bench_fast_big_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("FastBitReaderBig read 32 bits", |b| {
        b.iter(|| {
            let mut reader = FastBitReaderBig::new(Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits_fast(32).unwrap());
            }
        })
    });
}

fn bench_fast_little_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("FastBitReaderLittle read 32 bits", |b| {
        b.iter(|| {
            let mut reader = FastBitReaderLittle::new(Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits_fast(32).unwrap());
            }
        })
    });
}

fn bench_standard_big_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("StandardBitReader(BigEndian) read 32 bits", |b| {
        b.iter(|| {
            let mut reader = BitReader::with_byte_order(ByteOrder::BigEndian, Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits(32).unwrap());
            }
        })
    });
}

fn bench_standard_little_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("StandardBitReader(LittleEndian) read 32 bits", |b| {
        b.iter(|| {
            let mut reader =
                BitReader::with_byte_order(ByteOrder::LittleEndian, Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits(32).unwrap());
            }
        })
    });
}

fn bench_bulk_big_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("BulkBitReader(BigEndian) read 32 bits", |b| {
        b.iter(|| {
            let mut reader =
                BulkBitReader::with_endianness(ByteOrder::BigEndian, Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits(32).unwrap());
            }
        })
    });
}

fn bench_bulk_little_read_32(c: &mut Criterion) {
    let data = vec![0xFFu8; 4096];
    c.bench_function("BulkBitReader(LittleEndian) read 32 bits", |b| {
        b.iter(|| {
            let mut reader =
                BulkBitReader::with_endianness(ByteOrder::LittleEndian, Cursor::new(&data));
            for _ in 0..(data.len() / 4) {
                black_box(reader.read_bits(32).unwrap());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_fast_big_read_32,
    bench_fast_little_read_32,
    bench_standard_big_read_32,
    bench_standard_little_read_32,
    bench_bulk_big_read_32,
    bench_bulk_little_read_32,
);
criterion_main!(benches);
