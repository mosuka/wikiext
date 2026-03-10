# Benchmarks

Benchmarks for the wicket library using [Criterion](https://bheisler.github.io/criterion.rs/book/).

## Prerequisites

The `clean_wikitext` benchmark loads real articles from the test Wikipedia dump.
Download it into the `resources/` directory before running benchmarks:

```bash
curl -L https://dumps.wikimedia.org/testwiki/latest/testwiki-latest-pages-articles.xml.bz2 \
  -o resources/testwiki-latest-pages-articles.xml.bz2
```

## Running Benchmarks

Run all benchmarks:

```bash
cargo bench -p wicket --bench bench
```

Save a baseline before optimization work:

```bash
cargo bench -p wicket --bench bench -- --save-baseline before_optimization
```

Compare against a saved baseline:

```bash
cargo bench -p wicket --bench bench -- --baseline before_optimization
```

## HTML Reports

Criterion generates HTML reports under `target/criterion/`. Open
`target/criterion/report/index.html` in a browser to view graphs.
