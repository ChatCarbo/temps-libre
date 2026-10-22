# ⏳ Temps Libre

A minimalist and modern watch collection manager, written in Rust with [eframe / egui](https://github.com/emilk/egui).

## Features

- **Modern & Clean Interface**: Transparent Glassmorphism design, rounded corners, and fluid layout.
- **Multilingual**: Interface available in French, English, and Spanish (with language preference persistence).
- **Dual View**: 
  - **Cards View**: Minimalist and visual display (photo, logo, brand, and model).
  - **Table View**: Complete and detailed overview (dial color, case material, movement, drive type, diameter).
- **Instant Search**: Filter by brand, model, movement, or drive type.
- **Automatic Brand Logos**: Smart search and caching of brand logos (optimized for independent and micro-brand watches).
- **Standalone HTML Export**: Generate a single portable web page with all your photos embedded in base64.
- **Local Storage**: 100% local, private, and secure management using lightweight JSON files.

## Installation

Prerequisites: [Rust](https://rustup.rs) (2021 edition).

```bash
git clone [https://github.com/YOUR_USERNAME/temps-libre.git](https://github.com/YOUR_USERNAME/temps-libre.git)
cd temps-libre
cargo run --release