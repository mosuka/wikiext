# wikiext

A high-performance tool that extracts plain text from Wikipedia XML dump files.

wikiext is a Rust reimplementation of [wikiextractor](https://github.com/attardi/wikiextractor), offering significantly faster processing through parallel execution and efficient streaming.

## Features

- Streaming XML parsing that handles multi-gigabyte dumps without loading them into memory
- Parallel text extraction using multiple CPU cores via [rayon](https://crates.io/crates/rayon)
- Automatic bzip2 decompression for `.xml.bz2` dump files
- Output compatible with wikiextractor (doc format and JSON format)
- File splitting with configurable maximum size per file
- Namespace filtering to extract only specific page types

## Installation

### From crates.io

```sh
cargo install wikiext-cli
```

### From source

Requires Rust 1.85 or later.

```sh
git clone https://github.com/mosuka/wext.git
cd wext
cargo build --release
```

## Quick Start

```sh
# Download a Wikipedia dump
wget https://dumps.wikimedia.org/simplewiki/latest/simplewiki-latest-pages-articles.xml.bz2

# Extract plain text
wikiext simplewiki-latest-pages-articles.xml.bz2 -o output/

# JSON output
wikiext simplewiki-latest-pages-articles.xml.bz2 -o output/ --json

# Write to stdout
wikiext simplewiki-latest-pages-articles.xml.bz2 -o - -q | head -50
```

## CLI Options

| Option | Description | Default |
| ------ | ----------- | ------- |
| `<INPUT>` | Input Wikipedia XML dump file (`.xml` or `.xml.bz2`) | (required) |
| `-o, --output` | Output directory, or `-` for stdout | `text` |
| `-b, --bytes` | Maximum bytes per output file (e.g., `1M`, `500K`, `1G`) | `1M` |
| `-c, --compress` | Compress output files using bzip2 | `false` |
| `--json` | Write output in JSON format | `false` |
| `--processes` | Number of parallel workers | CPU count |
| `-q, --quiet` | Suppress progress output on stderr | `false` |
| `--namespaces` | Comma-separated namespace IDs to extract | `0` |

## Output Formats

### Doc Format (default)

```xml
<doc id="1" url="https://en.wikipedia.org/wiki/April" title="April">
April is the fourth month of the year...
</doc>
```

### JSON Format

```json
{"id":"1","url":"https://en.wikipedia.org/wiki/April","title":"April","text":"April is the fourth month of the year..."}
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
wikiext = "0.1.0"
```

```rust
use wikiext::{open_dump, clean_wikitext, format_page, make_url, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_dump("dump.xml.bz2".as_ref(), &[0])?;
    let url_base = reader.url_base().to_string();

    for result in reader.take(5) {
        let article = result?;
        let text = clean_wikitext(&article.text);
        let url = make_url(&url_base, &article.title);
        let output = format_page(
            article.id, &article.title, &url, &text, OutputFormat::Doc,
        );
        println!("{}", output);
    }

    Ok(())
}
```

## Documentation

- [English Documentation](https://mosuka.github.io/wext/en/)
- [日本語ドキュメント](https://mosuka.github.io/wext/ja/)
- [API Documentation (docs.rs)](https://docs.rs/wikiext)

## License

MIT OR Apache-2.0
