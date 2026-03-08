use crate::error::Error;

/// Output format for extracted Wikipedia pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Doc format: `<doc id="..." url="..." title="...">text</doc>`
    Doc,
    /// JSON format: one JSON object per line with id, url, title, and text fields.
    Json,
}

/// Escapes HTML special characters in the given string.
///
/// Replaces `&`, `<`, `>`, and `"` with their corresponding HTML entities.
///
/// # Arguments
///
/// * `s` - The string to escape.
///
/// # Returns
///
/// A new string with HTML special characters escaped.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Generates a Wikipedia URL from a base URL and a page title.
///
/// Spaces in the title are replaced with underscores, following Wikipedia URL conventions.
///
/// # Arguments
///
/// * `url_base` - The base URL (e.g., `https://ja.wikipedia.org/wiki`).
/// * `title` - The page title.
///
/// # Returns
///
/// The full URL string.
///
/// # Examples
///
/// ```
/// use wikiext::extractor::make_url;
///
/// let url = make_url("https://ja.wikipedia.org/wiki", "東京都");
/// assert_eq!(url, "https://ja.wikipedia.org/wiki/東京都");
/// ```
pub fn make_url(url_base: &str, title: &str) -> String {
    let encoded_title = title.replace(' ', "_");
    format!("{}/{}", url_base, encoded_title)
}

/// Formats a Wikipedia page into the specified output format.
///
/// # Arguments
///
/// * `id` - The page identifier.
/// * `title` - The page title.
/// * `url_base` - The base URL for constructing the page URL.
/// * `text` - The extracted plain text of the page.
/// * `format` - The output format to use (`Doc` or `Json`).
///
/// # Returns
///
/// A formatted string representation of the page.
///
/// For `Doc` format, produces:
/// ```text
/// <doc id="ID" url="URL" title="TITLE">
/// TEXT
/// </doc>
///
/// ```
///
/// For `Json` format, produces a single-line JSON object with a trailing newline.
pub fn format_page(
    id: u64,
    title: &str,
    url_base: &str,
    text: &str,
    format: OutputFormat,
) -> String {
    let url = make_url(url_base, title);

    match format {
        OutputFormat::Doc => {
            let escaped_title = escape_html(title);
            let escaped_url = escape_html(&url);
            format!(
                "<doc id=\"{}\" url=\"{}\" title=\"{}\">\n{}\n</doc>\n\n",
                id, escaped_url, escaped_title, text
            )
        }
        OutputFormat::Json => {
            let obj = serde_json::json!({
                "id": id.to_string(),
                "url": url,
                "title": title,
                "text": text,
            });
            // serde_json::to_string should not fail for simple string values.
            let json_str = serde_json::to_string(&obj).unwrap_or_default();
            format!("{}\n", json_str)
        }
    }
}

/// Parses a file size specification string into a byte count.
///
/// Supports the following suffixes (case-insensitive is NOT supported; use uppercase):
///
/// * `K` - Kilobytes (1024 bytes)
/// * `M` - Megabytes (1024^2 bytes)
/// * `G` - Gigabytes (1024^3 bytes)
///
/// A plain number without a suffix is interpreted as bytes.
/// A value of `0` means one article per file.
///
/// # Arguments
///
/// * `spec` - The file size specification string (e.g., `"1M"`, `"500K"`, `"1G"`, `"0"`).
///
/// # Returns
///
/// * `Ok(u64)` - The size in bytes.
/// * `Err(Error::InvalidFileSize)` - If the specification is invalid.
///
/// # Examples
///
/// ```
/// use wikiext::extractor::parse_file_size;
///
/// assert_eq!(parse_file_size("1M").unwrap(), 1048576);
/// assert_eq!(parse_file_size("500K").unwrap(), 512000);
/// assert_eq!(parse_file_size("0").unwrap(), 0);
/// ```
pub fn parse_file_size(spec: &str) -> Result<u64, Error> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err(Error::InvalidFileSize(spec.to_string()));
    }

    // Check for suffix
    let last_char = spec.chars().last().unwrap_or('0');
    let multiplier = match last_char {
        'K' => Some(1024u64),
        'M' => Some(1024u64 * 1024),
        'G' => Some(1024u64 * 1024 * 1024),
        _ => None,
    };

    match multiplier {
        Some(mult) => {
            let num_part = &spec[..spec.len() - 1];
            let num: u64 = num_part
                .parse()
                .map_err(|_| Error::InvalidFileSize(spec.to_string()))?;
            Ok(num * mult)
        }
        None => {
            // Plain number (bytes)
            let num: u64 = spec
                .parse()
                .map_err(|_| Error::InvalidFileSize(spec.to_string()))?;
            Ok(num)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_page_doc() {
        let result = format_page(
            1,
            "Test Page",
            "https://en.wikipedia.org/wiki",
            "Hello world.",
            OutputFormat::Doc,
        );
        let expected = "<doc id=\"1\" url=\"https://en.wikipedia.org/wiki/Test_Page\" title=\"Test Page\">\nHello world.\n</doc>\n\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_page_json() {
        let result = format_page(
            42,
            "Test Page",
            "https://en.wikipedia.org/wiki",
            "Some text here.",
            OutputFormat::Json,
        );
        let parsed: serde_json::Value = serde_json::from_str(result.trim()).unwrap();
        assert_eq!(parsed["id"], "42");
        assert_eq!(parsed["url"], "https://en.wikipedia.org/wiki/Test_Page");
        assert_eq!(parsed["title"], "Test Page");
        assert_eq!(parsed["text"], "Some text here.");
    }

    #[test]
    fn test_make_url_with_spaces() {
        let url = make_url("https://en.wikipedia.org/wiki", "New York City");
        assert_eq!(url, "https://en.wikipedia.org/wiki/New_York_City");
    }

    #[test]
    fn test_make_url_japanese_title() {
        let url = make_url("https://ja.wikipedia.org/wiki", "東京都");
        assert_eq!(url, "https://ja.wikipedia.org/wiki/東京都");
    }

    #[test]
    fn test_make_url_no_spaces() {
        let url = make_url("https://en.wikipedia.org/wiki", "Rust");
        assert_eq!(url, "https://en.wikipedia.org/wiki/Rust");
    }

    #[test]
    fn test_parse_file_size_kilobytes() {
        assert_eq!(parse_file_size("500K").unwrap(), 512000);
        assert_eq!(parse_file_size("1K").unwrap(), 1024);
    }

    #[test]
    fn test_parse_file_size_megabytes() {
        assert_eq!(parse_file_size("1M").unwrap(), 1048576);
        assert_eq!(parse_file_size("10M").unwrap(), 10485760);
    }

    #[test]
    fn test_parse_file_size_gigabytes() {
        assert_eq!(parse_file_size("1G").unwrap(), 1073741824);
    }

    #[test]
    fn test_parse_file_size_plain_number() {
        assert_eq!(parse_file_size("4096").unwrap(), 4096);
        assert_eq!(parse_file_size("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_file_size_invalid() {
        assert!(parse_file_size("").is_err());
        assert!(parse_file_size("abc").is_err());
        assert!(parse_file_size("M").is_err());
        assert!(parse_file_size("12X").is_err());
    }

    #[test]
    fn test_escape_html_in_doc_format() {
        let result = format_page(
            1,
            "A <b>bold</b> & \"quoted\" title",
            "https://en.wikipedia.org/wiki",
            "Some text.",
            OutputFormat::Doc,
        );
        assert!(result.contains("title=\"A &lt;b&gt;bold&lt;/b&gt; &amp; &quot;quoted&quot; title\""));
        assert!(result.contains("url=\"https://en.wikipedia.org/wiki/A_&lt;b&gt;bold&lt;/b&gt;_&amp;_&quot;quoted&quot;_title\""));
    }

    #[test]
    fn test_json_format_trailing_newline() {
        let result = format_page(
            1,
            "Title",
            "https://example.com",
            "Text",
            OutputFormat::Json,
        );
        assert!(result.ends_with('\n'));
        // Should be exactly one trailing newline (one line per article)
        assert!(!result.ends_with("\n\n"));
    }

    #[test]
    fn test_doc_format_trailing_newline() {
        let result = format_page(
            1,
            "Title",
            "https://example.com",
            "Text",
            OutputFormat::Doc,
        );
        // Doc format ends with </doc>\n\n (one blank line after)
        assert!(result.ends_with("</doc>\n\n"));
    }
}
