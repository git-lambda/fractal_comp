# Fractal Image Compression

A demo implementation of **fractal image compression** in Rust, featuring a CPU-based encoder and early experiments with GPU compute via WGSL and `wgpu`.

This project is primarily educational — exploring how fractal image encoding works under the hood — and is not intended for production use. Video explanation and theory can be found in: LINK_TO_BE_ADDED

---

## ✨ Features

- **CPU Encoder** – Fully functional (but *slow*), written in Rust.  
- **GPU Compute (WGSL)** – Work in progress, using `wgpu` for 60x+ speed increase.  
- CLI interface for compression tasks.  
- Modular code structure for experimentation.  

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (latest stable)
- `cargo` (comes with Rust installation)

### Build & Run
```bash
cargo run --release
