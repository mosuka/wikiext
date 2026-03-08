# Library API Overview

The `wicket` crate provides a Rust API for extracting plain text from Wikipedia XML dump files.

## Installation

```toml
[dependencies]
wicket = "0.1.0"
```

## Module Map

| Module | Primary Types | Purpose |
| ----- | ----- | ----- |
| `wicket::dump` | `Article`, `DumpReader`, `open_dump()` | Streaming XML dump parsing |
| `wicket::cleaner` | `clean_wikitext()` | Wikitext to plain text conversion |
| `wicket::extractor` | `OutputFormat`, `format_page()`, `make_url()` | Output formatting |
| `wicket::output` | `OutputConfig`, `OutputSplitter` | File splitting and rotation |
| `wicket::error` | `Error` | Error type definitions |

## Quick Example

```rust
use wicket::{open_dump, clean_wikitext, format_page, make_url, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_dump("dump.xml.bz2".as_ref(), &[0])?;
    let url_base = reader.url_base().to_string();

    for result in reader {
        let article = result?;
        let text = clean_wikitext(&article.text);
        let url = make_url(&url_base, &article.title);
        let output = format_page(
            article.id, &article.title, &url, &text, OutputFormat::Doc,
        );
        print!("{}", output);
    }

    Ok(())
}
```

## API Documentation

Full API documentation is available on [docs.rs/wicket](https://docs.rs/wicket).
