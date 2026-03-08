# Architecture Overview

wikiext is designed as a high-performance, streaming Wikipedia dump text extractor. It processes multi-gigabyte XML dumps by combining streaming I/O with batch-based parallelism.

## High-Level Data Flow

```text
Input (.xml / .xml.bz2)
  |
  v
DumpReader (streaming XML parse + namespace filter)
  |  yields Article { id, title, namespace, text }
  v
Batch (1000 articles)
  |
  v
rayon par_iter (parallel processing)
  |  clean_wikitext(text) -> plain text
  |  format_page(id, title, url_base, text, format) -> formatted string
  v
OutputSplitter (sequential write, file rotation)
  |
  v
Output files (AA/wiki_00, AA/wiki_01, ...)
```

## Design Principles

- **Streaming processing** -- XML is parsed as a stream; only one article is in memory at a time
- **Batch parallelism** -- CPU-bound wikitext cleaning is parallelized via rayon while I/O remains sequential
- **wikiextractor compatibility** -- output format and directory structure match the original Python tool
- **Fail-soft** -- malformed pages are logged and skipped rather than causing the entire process to abort
- **Library-first** -- core functionality lives in the `wikiext` library crate; the CLI is a thin wrapper
