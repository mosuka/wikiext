//! Benchmarks for the wicket library hot paths.
//!
//! The `clean_wikitext` benchmarks load real articles from the test Wikipedia
//! dump (`resources/testwiki-latest-pages-articles.xml.bz2`).
//! `format_page` and `make_url` benchmarks use small synthetic inputs to give
//! focused, stable measurements of those specific functions.
//!
//! Run all benchmarks:
//!   cargo bench -p wicket --bench bench
//!
//! Save a baseline before optimization work:
//!   cargo bench -p wicket --bench bench -- --save-baseline before_optimization
//!
//! Compare against the saved baseline:
//!   cargo bench -p wicket --bench bench -- --baseline before_optimization

use std::hint::black_box;
use std::path::Path;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use wicket::{Article, OutputFormat, clean_wikitext, format_page, make_url, open_dump};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Path to the test Wikipedia dump, relative to the wicket crate root.
const DUMP_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/testwiki-latest-pages-articles.xml.bz2"
);

/// Loads all main-namespace (ns=0) articles from the test dump.
///
/// Panics if the file cannot be opened, to surface setup problems early.
fn load_articles() -> Vec<Article> {
    let path = Path::new(DUMP_PATH);
    open_dump(path, &[0])
        .expect("failed to open test dump – does resources/testwiki-latest-pages-articles.xml.bz2 exist?")
        .filter_map(|r| r.ok())
        .collect()
}

// ── Synthetic inputs for format_page / make_url ───────────────────────────────

/// Title with no HTML-special characters (baseline for escape_html).
const PLAIN_TITLE: &str = "Albert Einstein";

/// Title containing HTML special characters that trigger escape_html.
const HTML_TITLE: &str = r#"A & B <script>alert("xss")</script> > test"#;

const URL_BASE: &str = "https://en.wikipedia.org/wiki";

// ── Benchmark groups ──────────────────────────────────────────────────────────

/// Benchmarks `clean_wikitext` on real Wikipedia articles loaded from the dump.
///
/// Reports both wall-clock time and throughput in input bytes per second so
/// that relative improvements after optimization are easy to compare.
fn bench_clean_wikitext(c: &mut Criterion) {
    let articles = load_articles();

    // Total bytes of raw wikitext across all articles.
    let total_bytes: u64 = articles.iter().map(|a| a.text.len() as u64).sum();
    let article_count = articles.len();

    let mut group = c.benchmark_group("clean_wikitext");
    group.throughput(Throughput::Bytes(total_bytes));

    // Benchmark: clean every article from the dump in a single iteration.
    // This is the most realistic measurement of real-world throughput.
    group.bench_function(
        BenchmarkId::new("dump", format!("{article_count}_articles")),
        |b| {
            b.iter(|| {
                for article in &articles {
                    black_box(clean_wikitext(black_box(&article.text)));
                }
            });
        },
    );

    group.finish();
}

/// Benchmarks `format_page` (Doc and JSON) with synthetic inputs.
///
/// `doc/html_title` exercises `escape_html` with special characters;
/// `doc/plain_title` shows the cost without escaping.
fn bench_format_page(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_page");

    // Doc format – no HTML escaping needed
    group.bench_function("doc/plain_title", |b| {
        b.iter(|| {
            format_page(
                black_box(1),
                black_box(PLAIN_TITLE),
                black_box(URL_BASE),
                black_box("Some plain text content for the article body."),
                black_box(OutputFormat::Doc),
            )
        });
    });

    // Doc format – triggers escape_html on title and URL
    group.bench_function("doc/html_title", |b| {
        b.iter(|| {
            format_page(
                black_box(1),
                black_box(HTML_TITLE),
                black_box(URL_BASE),
                black_box("Some plain text content for the article body."),
                black_box(OutputFormat::Doc),
            )
        });
    });

    // JSON format
    group.bench_function("json/plain_title", |b| {
        b.iter(|| {
            format_page(
                black_box(1),
                black_box(PLAIN_TITLE),
                black_box(URL_BASE),
                black_box("Some plain text content for the article body."),
                black_box(OutputFormat::Json),
            )
        });
    });

    group.finish();
}

/// Benchmarks `make_url` for various title types.
fn bench_make_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("make_url");

    group.bench_function("with_spaces", |b| {
        b.iter(|| make_url(black_box(URL_BASE), black_box("New York City")));
    });

    group.bench_function("without_spaces", |b| {
        b.iter(|| make_url(black_box(URL_BASE), black_box("Rust")));
    });

    group.bench_function("japanese_title", |b| {
        b.iter(|| {
            make_url(
                black_box("https://ja.wikipedia.org/wiki"),
                black_box("東京都"),
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_clean_wikitext,
    bench_format_page,
    bench_make_url
);
criterion_main!(benches);
