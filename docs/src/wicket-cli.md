# CLI Reference Overview

The `wicket` CLI extracts plain text from Wikipedia XML dump files.

## Usage

```sh
wicket [OPTIONS] <INPUT>
```

## Quick Reference

| Option | Description | Default |
| ----- | ----- | ----- |
| `<INPUT>` | Input Wikipedia XML dump file (`.xml` or `.xml.bz2`) | (required) |
| `-o, --output` | Output directory, or `-` for stdout | `text` |
| `-b, --bytes` | Maximum bytes per output file (e.g., `1M`, `500K`, `1G`) | `1M` |
| `-c, --compress` | Compress output files using bzip2 | `false` |
| `--json` | Write output in JSON format | `false` |
| `--processes` | Number of parallel workers | CPU count |
| `-q, --quiet` | Suppress progress output on stderr | `false` |
| `--namespaces` | Comma-separated namespace IDs to extract | `0` |

## Detailed Documentation

- [Options](wicket-cli/options.md) -- detailed description of all CLI options
- [Examples](wicket-cli/examples.md) -- common usage patterns and examples
