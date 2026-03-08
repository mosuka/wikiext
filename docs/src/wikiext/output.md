# output

The `output` module manages writing extracted articles to split output files following the wikiextractor directory naming convention.

## Types

### `OutputConfig`

Configuration for the output splitter.

| Field | Type | Description |
| ----- | ----- | ----- |
| `path` | `PathBuf` | Output directory path, or `"-"` for stdout |
| `max_file_size` | `u64` | Maximum bytes per output file |
| `compress` | `bool` | Whether to compress output with bzip2 |

### `OutputSplitter`

Manages file rotation and writing. Creates subdirectories and files as needed.

## Directory Naming Convention

Output files are organized using wikiextractor's naming convention:

```text
output/
  AA/
    wiki_00
    wiki_01
    ...
    wiki_99
  AB/
    wiki_00
    ...
```

- Each directory holds up to 100 files
- Directory names follow the pattern AA, AB, ..., AZ, BA, ..., ZZ
- When `compress` is enabled, files are named `wiki_00.bz2`, etc.

## Special Modes

- **stdout mode** -- When `path` is `"-"`, all output is written to stdout without splitting
- **Zero size** -- When `max_file_size` is 0, each article is written to its own file
