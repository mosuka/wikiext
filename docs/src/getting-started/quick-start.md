# Quick Start

## Obtaining a Wikipedia Dump

Download a Wikipedia dump from <https://dumps.wikimedia.org/>. For testing, the Simple English Wikipedia dump is recommended due to its small size:

```sh
wget https://dumps.wikimedia.org/simplewiki/latest/simplewiki-latest-pages-articles.xml.bz2
```

## CLI Quick Start

### Basic Extraction

Extract plain text from a Wikipedia dump:

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o output/
```

This reads the dump, extracts plain text from all main namespace articles, and writes the output to the `output/` directory in doc format, splitting files at 1 MB.

### JSON Output

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o output/ --json
```

### Write to stdout

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o - -q | head -50
```

## Library Quick Start

Here is a minimal Rust program that opens a dump and processes articles:

```rust
use wicket::{open_dump, clean_wikitext, format_page, make_url, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_dump("simplewiki-latest-pages-articles.xml.bz2".as_ref(), &[0])?;
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

## What's Next

- [CLI Reference](../wicket-cli.md) -- learn all CLI options
- [Architecture](../architecture/overview.md) -- understand how wicket works internally
