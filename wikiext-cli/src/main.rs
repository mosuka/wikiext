use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use wikiext::{clean_wikitext, format_page, open_dump, parse_file_size, OutputConfig, OutputFormat, OutputSplitter};

/// Extract plain text from Wikipedia XML dumps.
#[derive(Parser)]
#[command(name = "wikiext", version, about)]
struct Cli {
    /// Input Wikipedia XML dump file (.xml or .xml.bz2)
    input: PathBuf,

    /// Output directory (use '-' for stdout)
    #[arg(short, long, default_value = "text")]
    output: String,

    /// Maximum bytes per output file (e.g. 1M, 500K, 1G). 0 = one article per file
    #[arg(short, long, default_value = "1M")]
    bytes: String,

    /// Compress output files using bzip2
    #[arg(short, long)]
    compress: bool,

    /// Write output in JSON format
    #[arg(long)]
    json: bool,

    /// Number of parallel workers (defaults to number of CPUs)
    #[arg(long)]
    processes: Option<usize>,

    /// Suppress progress output
    #[arg(short, long)]
    quiet: bool,

    /// Comma-separated list of namespace IDs to extract (default: 0)
    #[arg(long, default_value = "0")]
    namespaces: String,
}

/// Batch size for parallel processing.
const BATCH_SIZE: usize = 1000;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Parse namespaces.
    let namespaces: Vec<i32> = cli
        .namespaces
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .with_context(|| format!("invalid namespace: '{s}'"))
        })
        .collect::<Result<Vec<_>>>()?;

    // Parse file size.
    let max_file_size =
        parse_file_size(&cli.bytes).with_context(|| format!("invalid bytes value: '{}'", cli.bytes))?;

    // Configure rayon thread pool.
    if let Some(n) = cli.processes {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .with_context(|| "failed to configure thread pool")?;
    }

    let output_format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Doc
    };

    // Open the dump.
    let mut dump_reader =
        open_dump(&cli.input, &namespaces).with_context(|| format!("failed to open dump: {:?}", cli.input))?;

    // We need to consume at least one item to initialize the reader and get url_base.
    // Collect articles in batches for parallel processing.
    let mut first_article = None;
    for result in dump_reader.by_ref() {
        match result {
            Ok(article) => {
                first_article = Some(article);
                break;
            }
            Err(e) => {
                eprintln!("warning: error reading page: {e}");
                continue;
            }
        }
    }

    let url_base = dump_reader.url_base().to_string();

    // Set up output.
    let config = OutputConfig {
        path: PathBuf::from(&cli.output),
        max_file_size,
        compress: cli.compress,
    };
    let mut output = OutputSplitter::new(config).with_context(|| "failed to create output")?;

    let mut total_pages: u64 = 0;

    // Process first article if we have one.
    if let Some(article) = first_article {
        let text = clean_wikitext(&article.text);
        let formatted = format_page(article.id, &article.title, &url_base, &text, output_format);
        output
            .write(&formatted)
            .with_context(|| "failed to write output")?;
        total_pages += 1;
    }

    // Process remaining articles in batches.
    loop {
        let batch: Vec<_> = dump_reader
            .by_ref()
            .filter_map(|result| match result {
                Ok(article) => Some(article),
                Err(e) => {
                    eprintln!("warning: error reading page: {e}");
                    None
                }
            })
            .take(BATCH_SIZE)
            .collect();

        if batch.is_empty() {
            break;
        }

        let batch_len = batch.len() as u64;

        // Parallel clean + format, preserving order.
        let results: Vec<String> = batch
            .par_iter()
            .map(|article| {
                let text = clean_wikitext(&article.text);
                format_page(article.id, &article.title, &url_base, &text, output_format)
            })
            .collect();

        // Write results in order (single writer thread).
        for formatted in &results {
            output
                .write(formatted)
                .with_context(|| "failed to write output")?;
        }

        total_pages += batch_len;

        if !cli.quiet {
            eprint!("\r{total_pages} pages processed");
        }
    }

    output.close().with_context(|| "failed to close output")?;

    if !cli.quiet {
        eprintln!("\r{total_pages} pages processed - done.");
    }

    Ok(())
}
