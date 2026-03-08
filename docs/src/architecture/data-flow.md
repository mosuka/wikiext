# Data Flow

This page describes how data flows through wikiext from input to output.

## Processing Pipeline

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

## Stage Details

### 1. XML Dump Reading

`DumpReader` uses `quick-xml` to parse the MediaWiki XML dump as a stream. For `.xml.bz2` files, the stream is automatically wrapped with `MultiBzDecoder` for transparent decompression.

The reader extracts:

- Page ID (`<id>` inside `<page>`, not inside `<revision>`)
- Title (`<title>`)
- Namespace (`<ns>`)
- Wikitext body (`<text>`)
- URL base from `<siteinfo><base>` (extracted once at startup)

Pages with namespaces not in the filter list are skipped at the iterator level.

### 2. Batch Collection

Articles are collected into batches of 1000 from the `DumpReader` iterator. This batch size balances parallelization overhead against memory usage.

### 3. Parallel Processing

Each batch is processed with `rayon::par_iter()`, which distributes work across CPU cores:

1. **`clean_wikitext(text)`** -- Converts wikitext markup to plain text. This is the most CPU-intensive step.
2. **`format_page(id, title, url, text, format)`** -- Formats the clean text into doc or JSON format.

Results are collected in order (rayon preserves element ordering with `par_iter`).

### 4. Sequential Output

Formatted strings are written sequentially to the `OutputSplitter`, which:

- Creates subdirectories (AA, AB, ..., ZZ) as needed
- Rotates to a new file after reaching the configured size limit
- Applies bzip2 compression when enabled
- Outputs to stdout when the path is `"-"`

## Parallelization Strategy

wikiext uses a batch-based parallelization approach rather than a pipeline with channels:

1. The main thread reads articles from the `DumpReader` in batches of 1000
2. Each batch is processed in parallel using `rayon::par_iter()`
3. Results are written sequentially to maintain deterministic output ordering
4. This repeats until all articles are processed

This approach is simple, maintains output ordering, and effectively parallelizes the CPU-bound cleaning step while keeping the I/O-bound reading and writing sequential.
