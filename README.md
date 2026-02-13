# 🚀 PASSWG: Extreme-Performance SIMD Generator

[English Description](README) | [Описание на русском](README_RU)

**PASSWG** is a low-level password generator written in Rust, specifically designed for maximum data throughput on x86_64 architectures using AVX2 instructions.

## 🎯 Why is it fast?

The program ignores standard slow string generation methods and works directly with CPU registers:

- **AVX2 / SIMD**: Generates and maps 32 symbols per clock cycle using 256-bit vectors.
    
- **Lock-Free Parallelism**: Thanks to the `Rayon` library, the workload is distributed across all available cores (P-cores and E-cores) without mutex bottlenecks.
    
- **Zero Modulo Bias**: Implements a rejection sampling algorithm to ensure perfect mathematical entropy (~6.52 bits per symbol).
    

## 📊 Benchmarks (Intel i3-12100f)

On a budget 4-core CPU, PASSWG delivers:

- **Speed**: ~485,000,000 passwords/sec (20 chars).
    
- **Throughput**: ~10.2 GB/s (RAM/Bus bottleneck).
    
- **Entropy**: ~130 bits for a 20-character password.
    

## 🛠 Usage

`passwg [length/words] [count] [flags] [other options]`

Example: `passwg 20 1 -w`. The flag `-w` automatically switches the logic from characters to words. You don't need to specify the number of words separately if they are already specified at the beginning.

Output formats include `--json` or `--csv`. For file output, use `-o <path>`. **IMPORTANT: You must manually add the .json or .csv extension.** Use `-s` for the built-in benchmark.

## ⚙️ Build

```
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release
```

## Important Details

Maximum speed is only achieved in **fast mode** `-f` and when writing to `/dev/null`. No standard SSD can handle ~10 GB/s, and console output is a major bottleneck.

## ⚠️ Disclaimer

The generator was developed with AI assistance. I am not planning to curate or support this project. **Fork it and play with it as you wish.** The project is considered complete as it fulfills its main purpose. All future fixes and features are your responsibility.
