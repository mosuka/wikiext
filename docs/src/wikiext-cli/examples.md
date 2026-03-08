# CLI Examples

## Basic Extraction

Extract text from a Wikipedia dump into the default `text/` directory:

```sh
wikiext simplewiki-latest-pages-articles.xml.bz2
```

## Custom Output Directory

```sh
wikiext dump.xml.bz2 -o output/
```

## Write to stdout

Pipe output directly to another command:

```sh
wikiext dump.xml.bz2 -o - -q | wc -l
```

## JSON Output with Compression

```sh
wikiext dump.xml.bz2 -o output/ --json -c
```

## Extract Talk Pages

Extract namespace 1 (talk pages) with 8 workers:

```sh
wikiext dump.xml.bz2 -o output/ --namespaces 1 --processes 8
```

## Multiple Namespaces

Extract main articles and user pages:

```sh
wikiext dump.xml.bz2 -o output/ --namespaces 0,2
```

## Small Output Files

Split output into 500 KB files:

```sh
wikiext dump.xml.bz2 -o output/ -b 500K
```

## One Article per File

```sh
wikiext dump.xml.bz2 -o output/ -b 0
```

## Output Directory Structure

After extraction, the output directory looks like:

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

With `--compress`:

```text
output/
  AA/
    wiki_00.bz2
    wiki_01.bz2
    ...
```
