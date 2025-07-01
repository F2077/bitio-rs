# bitio-rs

🚀A lightweight Rust library for bit-level I/O: read, write, peek, with both big-endian and little-endian support.

## Features

- Read arbitrary-length bit fields from any `Read` source (1–64 bits)
- Write arbitrary-length bit fields to any `Write` sink
- Peek bits without consuming them
- Fully endian-aware (BigEndian / LittleEndian)
- Two performance tiers:
  - **Standard**: Safe, validated standard implementation
  - **Fast**: 18-21x faster for performance-critical use
- minimal dependencies

## Installation

Add the following to your Cargo.toml:

```
[dependencies]
bitio-rs = "0.1.0"
```

## Quickstart

See [quickstart.rs](examples/quickstart.rs)

## Choosing an Implementation

- Standard BitReader:
  - Full error checking
  - Recommended for general use

- BulkBitReader
  - For bulk read
  - Slower than the standard version

- *FastBitReader*:
  - 18-21x faster (see benchmarks)
  - *Use at your own risk*

**Performance Comparison**

Benchmarks measured on Apple M4 (16GB RAM):

| Implementation                | Median Time | Relative Speed  | Notes            |
|-------------------------------|-------------|-----------------|------------------|
| (Standard) BitReader (Big)    | 5.3282 μs   | Baseline (1.0x) |                  |
| (Standard) BitReader (Little) | 5.5700 μs   | 0.96x           |                  |
| BulkBitReader (Big)           | 15.009 μs   | 2.82x slower    |                  |
| BulkBitReader (Little)        | 15.432 μs   | 2.77x slower    |                  |
| *FastBitReaderBig*            | 295.46 ns   | 18.0x faster    | incompatible API |
| *FastBitReaderLittle*         | 264.06 ns   | 21.1x faster    | incompatible API |

> **Performance vs Compatibility Tradeoff**: Fast implementations achieve 10-21x speedup but:
> - Incompatible API
> - Recommended only for performance-critical sections