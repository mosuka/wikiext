# CLI Options

## Input

```sh
wikiext <INPUT>
```

The input file is a positional argument. It must be a Wikipedia XML dump file, either uncompressed (`.xml`) or bzip2-compressed (`.xml.bz2`). Compression is detected automatically by file extension.

## Output Directory

```sh
wikiext dump.xml.bz2 -o output/
wikiext dump.xml.bz2 -o -
```

`-o, --output <PATH>` -- Specifies the output directory. Defaults to `text`.

- When set to a directory path, output files are created in the wikiextractor naming convention (AA/wiki_00, etc.)
- When set to `-`, all output is written to stdout without file splitting

## File Size

```sh
wikiext dump.xml.bz2 -b 500K
wikiext dump.xml.bz2 -b 1M
wikiext dump.xml.bz2 -b 1G
wikiext dump.xml.bz2 -b 0
```

`-b, --bytes <SIZE>` -- Maximum bytes per output file. Defaults to `1M`.

Supported suffixes: `K` (kilobytes), `M` (megabytes), `G` (gigabytes). When set to `0`, each article is written to its own file.

## Compression

```sh
wikiext dump.xml.bz2 -c
```

`-c, --compress` -- Compress output files using bzip2. Output files will have a `.bz2` extension.

## JSON Output

```sh
wikiext dump.xml.bz2 --json
```

`--json` -- Write output in JSON Lines format (one JSON object per line) instead of the default doc format.

## Parallel Workers

```sh
wikiext dump.xml.bz2 --processes 8
```

`--processes <N>` -- Number of parallel workers for text cleaning. Defaults to the number of CPU cores.

## Quiet Mode

```sh
wikiext dump.xml.bz2 -q
```

`-q, --quiet` -- Suppress progress output on stderr. Useful when piping output to another command.

## Namespace Filtering

```sh
wikiext dump.xml.bz2 --namespaces 0
wikiext dump.xml.bz2 --namespaces 0,1,2
```

`--namespaces <IDS>` -- Comma-separated list of namespace IDs to extract. Defaults to `0` (main articles only).

Common namespace IDs:

| ID | Namespace |
| ----- | ----- |
| 0 | Main (articles) |
| 1 | Talk |
| 2 | User |
| 3 | User talk |
| 4 | Wikipedia |
| 6 | File |
| 10 | Template |
| 14 | Category |
