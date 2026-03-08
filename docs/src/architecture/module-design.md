# Module Design

The `wicket` library crate is organized into five modules, each with a clear responsibility.

## Module Overview

| Module | Primary Types | Purpose |
| ----- | ----- | ----- |
| `dump` | `Article`, `DumpReader` | Streaming XML dump parsing |
| `cleaner` | `clean_wikitext()` | Wikitext to plain text conversion |
| `extractor` | `OutputFormat`, `format_page()` | Output formatting (doc/JSON) |
| `output` | `OutputConfig`, `OutputSplitter` | File splitting and rotation |
| `error` | `Error` | Error type definitions |

## Module Details

### `dump` -- XML Dump Reader

Streaming XML parser built on `quick-xml`. Reads MediaWiki XML dump files and yields `Article` structs.

- **`Article`** -- A single Wikipedia page with `id` (u64), `title` (String), `namespace` (i32), and `text` (String)
- **`DumpReader<R: BufRead>`** -- Iterator that streams articles from an XML source with namespace filtering
- **`open_dump(path, namespaces)`** -- Opens a dump file with automatic `.bz2` detection using `MultiBzDecoder`

The reader parses `<siteinfo><base>` to extract the wiki's URL base, which is exposed via `url_base()`.

### `cleaner` -- Wikitext Cleaner

Converts MediaWiki markup into plain text using a two-stage approach:

1. **AST-based cleaning** -- Uses `parse_wiki_text_2` to build an AST and walks text nodes
2. **Regex fallback** -- When AST parsing fails, falls back to regex-based cleanup

Key function: `clean_wikitext(wikitext: &str) -> String`

### `extractor` -- Output Formatter

Formats extracted articles into the output representation.

- **`OutputFormat`** -- Enum with `Doc` and `Json` variants
- **`format_page(id, title, url, text, format)`** -- Formats a single article
- **`make_url(url_base, title)`** -- Constructs a Wikipedia article URL
- **`parse_file_size(spec)`** -- Parses size specifications like `1M`, `500K`, `1G`

### `output` -- File Splitter

Manages writing extracted articles to split output files following the wikiextractor directory naming convention.

- **`OutputConfig`** -- Configuration for output path, max file size, and compression
- **`OutputSplitter`** -- Manages file rotation with AA/wiki_00 naming (100 files per directory, directories AA through ZZ)

Supports stdout output (path = `"-"`), bzip2 compression, and configurable file size limits.

### `error` -- Error Types

Defines the `Error` enum using `thiserror`:

- `Io` -- I/O errors
- `XmlReader` -- XML parsing errors from `quick-xml`
- `JsonSerialization` -- JSON serialization errors

## Public Exports

The library's `lib.rs` re-exports key types for convenience:

```rust
pub use cleaner::clean_wikitext;
pub use dump::{open_dump, Article, DumpReader};
pub use error::Error;
pub use extractor::{format_page, make_url, parse_file_size, OutputFormat};
pub use output::{OutputConfig, OutputSplitter};
```
