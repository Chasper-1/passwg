# 🚀 PASSWG: Blazing Fast SIMD Generator

[English Description](README.md) | [Описание на русском](README_RU.md)

**PASSWG** is a low-level password generator written in Rust, engineered to hit the absolute limits of data throughput on x86_64 architectures.

## 🎯 Why is it so fast?

The program bypasses standard, slow string allocation methods and operates directly with CPU registers:

- **AVX2 / SIMD**: Generates and maps 32 characters per clock cycle using 256-bit vectors.
    
- **Lock-Free Parallelism**: Powered by the `Rayon` library, the workload is distributed across all available cores (P-cores and E-cores) without mutex bottlenecks.
    
- **Zero Modulo Bias**: Implements a rejection sampling algorithm to ensure perfect mathematical entropy (~6.52 bits per symbol).
    

## 📊 Benchmarks (Intel i3-12100f)

On a budget 4-core CPU, PASSWG delivers:

- **Speed**: ~485,000,000 passwords/sec (20 chars).
    
- **Throughput**: ~10.2 GB/s (RAM/Bus bottleneck).
    
- **Entropy**: ~130 bits for a 20-character password.
    

<details> <summary>View Benchmark Screenshot</summary> <img src="" alt="benchmark"> </details>

## 🛠 Features

- **Three ChaCha Modes**: Choose between ChaCha8, 12, or 20 rounds (`-r`).
    
- **Fast Mode (`-f`)**: Maximum optimization for the `[A-Za-z0-0_-]` character set.
    
- **Word Mode (`-w`)**: Generates readable phrases.
    
- **Output Formats**: Plain text, JSON, CSV.
    
- **Clipboard Support**: Direct pipe to Wayland clipboard (`-c`).
    

## Usage

`passwg [length/words] [count] [flags] [options]`

If you want to generate passwords as words, the logic remains the same: `passwg 20 1 -w`. The flag position is flexible; `passwg -w 20 1` will yield the same result. However, I recommend sticking to the standard syntax shown above. When the `-w` flag is active, the initial arguments automatically switch from character count to word count.

**Output:** You can use `--json` or `--csv` formats, or keep it as plain text. To save to a file, use the `-o` flag followed by the path. If the file doesn't exist, it will be created automatically. **IMPORTANT: You must manually include the .json or .csv extension in the filename.**

**Built-in Benchmark:** Use the `-s` flag to track speed and performance metrics in real-time.

## ⚙️ Build

<details> <summary>Or use aggressive hardware-specific optimizations:</summary>

_Note: The 2-second benchmark result was achieved using these exact compilation flags._

</details>

## Important Details

Maximum speed is strictly achieved in **fast mode** (`-f`). If your generation speed is significantly lower, the bottleneck is likely:

1. Your hardware.
    
2. You didn't use the `target-cpu=native` flag during compilation.
    
3. You forgot to enable `-f` mode.
    

Also, peak throughput was measured while writing to `/dev/null`.

<details> <summary>Why /dev/null?</summary> Because no consumer SSD on Earth can keep up with a 10 GB/s stream. Writing to a disk will throttle the generator. Similarly, printing everything to the console is EXTREMELY slow—don't do it if you're chasing records. </details>

# Heads Up!!!

This generator was heavily assisted by AI during development. For this reason, I am not planning to actively maintain or curate this project. **Fork it, play with it, break it.** I’m sharing this as a tool that I personally needed and as a showcase of what AI-assisted development can achieve.

The project is considered "feature-complete" because it fulfills its primary purpose: generating passwords. Any further polish or features are up to your imagination. If you encounter bugs, you're on your own, though it should work perfectly as-is.

# Additional Info

- If you have questions, feel free to open an issue here; I'll try to reply when possible.
    
- This isn't an ad for AI tools. I built this for myself, it turned out solid, so I decided to share.
    
- If the code looks "wrong" or "unclean" to you—make it better yourself. It's a ready-to-use base, even if it's not "perfect."
    

# Over and out!