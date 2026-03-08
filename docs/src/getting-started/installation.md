# Installation

## Prerequisites

- **Rust 1.85 or later** (stable channel) from [rust-lang.org](https://www.rust-lang.org/)
- **Cargo** (Rust's package manager, included with Rust)

## Installing the CLI Tool

### From crates.io

```sh
cargo install wicket-cli
```

### From Source

```sh
git clone https://github.com/mosuka/wext.git
cd wext
cargo build --release
```

The binary will be available at `./target/release/wicket`.

Verify the installation:

```sh
./target/release/wicket --help
```

## Using as a Library

Add wicket to your project's `Cargo.toml`:

```toml
[dependencies]
wicket = "0.1.0"
```

## Supported Platforms

wicket is tested on the following platforms:

| OS | Architecture |
| ----- | ----- |
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), aarch64 (Apple Silicon) |
| Windows | x86_64, aarch64 |
