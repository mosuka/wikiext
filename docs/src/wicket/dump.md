# dump

The `dump` module provides streaming XML parsing of MediaWiki dump files.

## Types

### `Article`

A single Wikipedia page extracted from the dump.

| Field | Type | Description |
| ----- | ----- | ----- |
| `id` | `u64` | Page ID |
| `title` | `String` | Page title |
| `namespace` | `i32` | Namespace ID (0 = main articles) |
| `text` | `String` | Raw wikitext content |

### `DumpReader<R: BufRead>`

An iterator that streams `Article` values from an XML dump source.

- Implements `Iterator<Item = Result<Article, Error>>`
- Filters articles by namespace at the iterator level
- Exposes `url_base()` to retrieve the wiki's base URL from `<siteinfo>`

## Functions

### `open_dump(path: &Path, namespaces: &[i32]) -> Result<DumpReader<...>>`

Opens a Wikipedia XML dump file for reading.

- Automatically detects `.bz2` extension and applies `MultiBzDecoder`
- Parses `<siteinfo>` to extract the URL base
- Configures namespace filtering

## Usage

```rust
use wicket::open_dump;

let reader = open_dump("dump.xml.bz2".as_ref(), &[0])?;
println!("URL base: {}", reader.url_base());

for result in reader {
    let article = result?;
    println!("[{}] {}", article.id, article.title);
}
```
