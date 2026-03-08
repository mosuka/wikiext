# Workspace Structure

wicket is organized as a **Cargo workspace** with two crates and supporting directories.

## Directory Layout

```text
wext/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock              # Dependency lock file
├── LICENSE                 # MIT OR Apache-2.0
├── README.md               # Project overview
├── wicket/                # Core library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Module declarations and re-exports
│       ├── dump.rs         # XML dump streaming parser
│       ├── cleaner.rs      # Wikitext to plain text conversion
│       ├── extractor.rs    # Output formatting (doc/JSON)
│       ├── output.rs       # File splitting and rotation
│       └── error.rs        # Error types
├── wicket-cli/            # CLI binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # CLI entry point
├── docs/                   # mdBook documentation (this book)
│   ├── book.toml
│   ├── src/
│   └── ja/                 # Japanese documentation
│       ├── book.toml
│       └── src/
└── .github/
    └── workflows/          # CI/CD pipelines
        ├── regression.yml  # Test on push/PR
        ├── release.yml     # Release builds and publishing
        ├── periodic.yml    # Weekly stability tests
        └── deploy-docs.yml # Documentation deployment
```

## Crate Details

### `wicket` (Core Library)

The core library provides streaming XML parsing, wikitext cleaning, output formatting, and file splitting.

| Dependency | Version | Purpose |
| ----- | ----- | ----- |
| `quick-xml` | 0.39 | Streaming XML parsing |
| `parse-wiki-text-2` | 0.2 | Wikitext AST parsing |
| `regex` | 1.12 | Fallback wikitext cleaning |
| `bzip2` | 0.6 | Bzip2 compression/decompression |
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON output formatting |
| `rayon` | 1.11 | Data parallelism (used by CLI) |
| `thiserror` | 2.0 | Error type derivation |
| `log` | 0.4 | Logging facade |

### `wicket-cli` (CLI Binary)

The CLI provides a command-line interface to wicket's functionality.

| Dependency | Version | Purpose |
| ----- | ----- | ----- |
| `clap` | 4.5 | Command-line argument parsing |
| `rayon` | 1.11 | Parallel batch processing |
| `bzip2` | 0.6 | Compressed output support |
| `env_logger` | 0.11 | Logging output |
| `anyhow` | 1.0 | Error handling in binary |
| `wicket` | 0.1 | Core library (workspace member) |

## Workspace Configuration

The workspace uses Cargo resolver version 3 (Rust Edition 2024):

```toml
[workspace]
resolver = "3"
members = ["wicket", "wicket-cli"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
```

Shared dependencies are defined at the workspace level in `[workspace.dependencies]` and referenced by each crate with `{ workspace = true }`.
