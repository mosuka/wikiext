# Introduction

**wicket** is a high-performance tool that extracts plain text from Wikipedia XML dump files. It is a Rust reimplementation of [wikiextractor](https://github.com/attardi/wikiextractor), offering significantly faster processing through parallel execution and efficient streaming.

## Key Features

- **Streaming XML parsing** -- handles multi-gigabyte dumps without loading them into memory
- **Parallel text extraction** -- uses multiple CPU cores via [rayon](https://crates.io/crates/rayon)
- **Automatic bzip2 decompression** -- transparently handles `.xml.bz2` dump files
- **wikiextractor-compatible output** -- both doc format and JSON format
- **File splitting** -- configurable maximum size per output file
- **Namespace filtering** -- extract only specific page types (main articles, talk pages, etc.)

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

## Current Version

wicket v0.1.0 -- Rust Edition 2024, minimum Rust version 1.85.

## Links

- [GitHub Repository](https://github.com/mosuka/wext)
- [crates.io](https://crates.io/crates/wicket)
- [API Documentation (docs.rs)](https://docs.rs/wicket)
- [Japanese Documentation (日本語)](../ja/)
