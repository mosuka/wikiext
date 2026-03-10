/// Wikitext cleaner module.
///
/// Converts MediaWiki markup (wikitext) into plain text by parsing the wikitext
/// into an AST using `parse_wiki_text_2` and extracting text content. Falls back
/// to regex-based cleaning when AST parsing fails.
///
/// The parser is configured with both English and Japanese Wikipedia namespaces
/// so that it can correctly handle dumps from either language edition without
/// changing the public API.
use std::sync::LazyLock;

use log::warn;
use parse_wiki_text_2::{Configuration, ConfigurationSource, Node};
use regex::Regex;

/// Pre-built parser configuration that recognises both English and Japanese
/// Wikipedia namespaces (Category/カテゴリ, File/Image/ファイル, REDIRECT/転送).
static WIKI_CONFIG: LazyLock<Configuration> = LazyLock::new(|| {
    Configuration::new(&ConfigurationSource {
        category_namespaces: &["category", "カテゴリ"],
        extension_tags: &[
            "categorytree",
            "ce",
            "charinsert",
            "chem",
            "gallery",
            "graph",
            "hiero",
            "imagemap",
            "indicator",
            "inputbox",
            "mapframe",
            "maplink",
            "math",
            "nowiki",
            "poem",
            "pre",
            "ref",
            "references",
            "score",
            "section",
            "source",
            "syntaxhighlight",
            "templatedata",
            "timeline",
        ],
        file_namespaces: &["file", "image", "ファイル"],
        link_trail: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        magic_words: &[
            "DISAMBIG",
            "FORCETOC",
            "HIDDENCAT",
            "INDEX",
            "NEWSECTIONLINK",
            "NOCC",
            "NOCOLLABORATIONHUBTOC",
            "NOCONTENTCONVERT",
            "NOEDITSECTION",
            "NOGALLERY",
            "NOGLOBAL",
            "NOINDEX",
            "NONEWSECTIONLINK",
            "NOTC",
            "NOTITLECONVERT",
            "NOTOC",
            "STATICREDIRECT",
            "TOC",
        ],
        protocols: &[
            "//",
            "bitcoin:",
            "ftp://",
            "ftps://",
            "geo:",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc://",
            "ircs://",
            "magnet:",
            "mailto:",
            "mms://",
            "news:",
            "nntp://",
            "redis://",
            "sftp://",
            "sip:",
            "sips:",
            "sms:",
            "ssh://",
            "svn://",
            "tel:",
            "telnet://",
            "urn:",
            "worldwind://",
            "xmpp:",
        ],
        redirect_magic_words: &["REDIRECT", "転送"],
    })
});

/// Tags whose content should be completely removed during text extraction.
const SKIP_TAGS: &[&str] = &[
    "ref",
    "references",
    "gallery",
    "source",
    "syntaxhighlight",
    "nowiki",
    "code",
    "math",
];

// Regex patterns for fallback cleaning, compiled once using LazyLock.

/// Matches `{{...}}` templates, including nested ones.
static RE_TEMPLATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{[^{}]*\}\}").expect("invalid regex"));

/// Matches `[[Category:...]]` and `[[カテゴリ:...]]` links.
static RE_CATEGORY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[(?:Category|カテゴリ):[^\]]*\]\]").expect("invalid regex"));

/// Matches `[[File:...]]`, `[[Image:...]]`, and `[[ファイル:...]]` links.
static RE_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[(?:File|Image|ファイル):[^\]]*\]\]").expect("invalid regex"));

/// Matches `[[target|text]]` piped links and captures the display text.
static RE_PIPED_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[[^\]|]+\|([^\]]+)\]\]").expect("invalid regex"));

/// Matches `[[target]]` simple links and captures the target.
static RE_SIMPLE_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]|]+)\]\]").expect("invalid regex"));

/// Matches `[url text]` external links and captures the display text.
static RE_EXTERNAL_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[https?://[^\s\]]+ ([^\]]+)\]").expect("invalid regex"));

/// Matches `'''text'''` bold markup.
static RE_BOLD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"'''([^']+)'''").expect("invalid regex"));

/// Matches `''text''` italic markup.
static RE_ITALIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"''([^']+)''").expect("invalid regex"));

/// Matches `== heading ==` markup (any level).
static RE_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"=={1,6}\s*(.+?)\s*=={1,6}").expect("invalid regex"));

/// Matches HTML tags (opening and closing).
static RE_HTML_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>").expect("invalid regex"));

/// Matches `<ref>...</ref>` and `<ref .../>` tags.
static RE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<ref[^>]*(?:>[\s\S]*?</ref>|/>)").expect("invalid regex"));

/// Matches `{|...|}`wiki tables.
static RE_TABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{\|.*?\|\}").expect("invalid regex"));

// Post-processing patterns to catch markup remnants that survive AST/fallback cleaning.

/// Converts wikitext markup into plain text.
///
/// Parses the given wikitext using `parse_wiki_text_2` to build an AST, then
/// extracts plain text from the AST nodes. If parsing fails, falls back to
/// regex-based cleaning.
///
/// # Arguments
///
/// * `wikitext` - The raw wikitext markup string to clean.
///
/// # Returns
///
/// A `String` containing the extracted plain text.
pub fn clean_wikitext(wikitext: &str) -> String {
    let result = WIKI_CONFIG.parse(wikitext);

    match result {
        Ok(output) => {
            // Wikitext shrinks significantly after markup removal; half the
            // input length is a reasonable initial capacity that avoids most
            // reallocations without over-allocating.
            let mut text = String::with_capacity(wikitext.len() / 2);
            extract_text_from_nodes(&output.nodes, &mut text);
            clean_text(&text)
        }
        Err(_) => {
            warn!("Failed to parse wikitext with AST parser, using fallback cleaner");
            let cleaned = fallback_clean(wikitext);
            clean_text(&cleaned)
        }
    }
}

/// Recursively extracts plain text from a slice of AST nodes.
///
/// Walks the AST produced by `parse_wiki_text_2` and appends readable text
/// content to `output`, applying the extraction rules described in the module
/// documentation.
///
/// # Arguments
///
/// * `nodes` - The slice of AST nodes to process.
/// * `output` - The mutable string buffer to append extracted text to.
#[inline]
fn extract_text_from_nodes(nodes: &[Node], output: &mut String) {
    for node in nodes {
        match node {
            Node::Text { value, .. } => {
                output.push_str(value);
            }
            Node::CharacterEntity { character, .. } => {
                output.push(*character);
            }
            Node::Heading { nodes, .. } => {
                output.push('\n');
                extract_text_from_nodes(nodes, output);
                output.push('\n');
            }
            Node::Link { text, target, .. } => {
                if text.is_empty() {
                    output.push_str(target);
                } else {
                    extract_text_from_nodes(text, output);
                }
            }
            Node::ExternalLink { nodes, .. } => {
                // Extract only the label text, skipping URL nodes.
                // The AST may represent the external link content as a
                // single Text node containing "URL label_text", so we
                // strip the leading URL portion.
                for n in nodes {
                    match n {
                        Node::Text { value, .. } => {
                            if value.starts_with("http://") || value.starts_with("https://") {
                                // URL followed by optional label after the first space.
                                if let Some(pos) = value.find(' ') {
                                    output.push_str(value[pos + 1..].trim());
                                }
                            } else {
                                output.push_str(value);
                            }
                        }
                        _ => {
                            extract_text_from_nodes(std::slice::from_ref(n), output);
                        }
                    }
                }
            }
            Node::Bold { .. } | Node::Italic { .. } | Node::BoldItalic { .. } => {
                // These are just markers; actual text is in separate Text nodes
            }
            Node::Template { .. }
            | Node::Category { .. }
            | Node::Image { .. }
            | Node::Table { .. }
            | Node::Comment { .. }
            | Node::MagicWord { .. }
            | Node::Parameter { .. }
            | Node::Redirect { .. }
            | Node::StartTag { .. }
            | Node::EndTag { .. } => {
                // Skip entirely
            }
            Node::Tag { name, nodes, .. } => {
                if !SKIP_TAGS.contains(&name.as_ref()) {
                    extract_text_from_nodes(nodes, output);
                }
            }
            Node::ParagraphBreak { .. } => {
                output.push_str("\n\n");
            }
            Node::HorizontalDivider { .. } => {
                output.push('\n');
            }
            Node::UnorderedList { items, .. } | Node::OrderedList { items, .. } => {
                for item in items {
                    extract_text_from_nodes(&item.nodes, output);
                    output.push('\n');
                }
            }
            Node::DefinitionList { items, .. } => {
                for item in items {
                    extract_text_from_nodes(&item.nodes, output);
                    output.push('\n');
                }
            }
            Node::Preformatted { nodes, .. } => {
                extract_text_from_nodes(nodes, output);
            }
        }
    }
}

/// Removes HTML comment remnants (`!--...--`) from a string.
///
/// Reproduces the behaviour of the regex `!--.*?--` (non-greedy): each
/// `!--` is paired with the nearest following `--` and the entire span is
/// deleted.
///
/// # Arguments
///
/// * `s` - A string slice known to contain at least one `!--`.
///
/// # Returns
///
/// A new `String` with all comment remnants removed.
#[inline]
fn strip_comment_remnants(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(start) = rest.find("!--") {
        result.push_str(&rest[..start]);
        // Advance past `!--` and look for the closing `--`.
        let after_open = &rest[start + 3..];
        if let Some(end) = after_open.find("--") {
            rest = &after_open[end + 2..];
        } else {
            // No closing `--`; discard the rest (matches regex behaviour).
            return result;
        }
    }

    result.push_str(rest);
    result
}

/// Removes orphaned template closing sequences from a line.
///
/// Reproduces the behaviour of the regex `[^{]*\}\}` used in `replace_all`:
/// every maximal run of non-`{` characters followed by `}}` is deleted.
///
/// # Arguments
///
/// * `s` - A string slice known to contain at least one `}}`.
///
/// # Returns
///
/// A new `String` with orphaned template closes removed.
#[inline]
fn strip_orphaned_template_close(s: &str) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Look ahead for `}}`.
        if i + 1 < len && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            // Walk backwards through result to remove preceding non-`{` bytes.
            while let Some(&last) = result.last() {
                if last == b'{' {
                    break;
                }
                result.pop();
            }
            // Skip the `}}`.
            i += 2;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    // SAFETY: We only remove ASCII bytes (`}` = 0x7D and non-`{` ASCII) from
    // a valid UTF-8 sequence, which always yields valid UTF-8.
    unsafe { String::from_utf8_unchecked(result) }
}

/// Collapses runs of two or more ASCII spaces into a single space.
///
/// Scans the input as bytes (spaces are always `0x20` regardless of the
/// surrounding Unicode characters) and writes to a new `String` only when
/// consecutive spaces are found.  This avoids the overhead of a full regex
/// DFA while remaining correct for any UTF-8 input.
///
/// # Arguments
///
/// * `s` - A string slice that is known to contain at least one run of two
///   or more consecutive spaces (i.e. `s.contains("  ")` is true).
///
/// # Returns
///
/// A new `String` with all multi-space runs replaced by a single space.
#[inline]
fn collapse_spaces(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(s.len());
    let mut last_was_space = false;
    for &b in bytes {
        if b == b' ' {
            if !last_was_space {
                result.push(b);
            }
            last_was_space = true;
        } else {
            result.push(b);
            last_was_space = false;
        }
    }
    // SAFETY: Removing duplicate space bytes (0x20, single-byte ASCII) from a
    // valid UTF-8 byte sequence always yields a valid UTF-8 byte sequence.
    unsafe { String::from_utf8_unchecked(result) }
}

/// Post-processes extracted text by removing markup remnants and normalizing
/// whitespace.
///
/// Performs the following cleanup steps:
/// 1. Removes orphaned template closing braces (`}}`) and preceding parameter text
/// 2. Removes lines consisting solely of template parameter syntax (`|key = value`)
/// 3. Removes HTML comment remnants where angle brackets were already stripped
/// 4. Trims leading and trailing whitespace from each line
/// 5. Collapses consecutive spaces into a single space
/// 6. Collapses three or more consecutive newlines into two (one blank line)
/// 7. Trims leading and trailing whitespace from the entire result
///
/// # Arguments
///
/// * `text` - The raw extracted text to clean up.
///
/// # Returns
///
/// A `String` with markup remnants removed and whitespace normalized.
#[inline]
fn clean_text(text: &str) -> String {
    // Single-pass approach: write directly into a pre-allocated buffer instead
    // of collecting into Vec<String> and then joining.  Blank-line runs are
    // tracked with a counter so that RE_MULTI_NEWLINE is no longer needed.
    let mut result = String::with_capacity(text.len());
    // Number of consecutive blank (or skipped) lines since the last content line.
    let mut blank_run: usize = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        // Fast path: skip lines that start with `|`.  In cleaned wikitext
        // these are invariably template parameter remnants or table row
        // fragments, both of which should be discarded.
        if trimmed.starts_with('|') {
            blank_run += 1;
            continue;
        }

        // Remove orphaned template closes: strips every span of non-`{`
        // characters followed by `}}` without DFA overhead.
        let s1: std::borrow::Cow<str> = if trimmed.contains("}}") {
            std::borrow::Cow::Owned(strip_orphaned_template_close(trimmed))
        } else {
            std::borrow::Cow::Borrowed(trimmed)
        };
        // Remove HTML comment remnants (`!--...--`) without a regex engine.
        let s2: std::borrow::Cow<str> = if s1.contains("!--") {
            std::borrow::Cow::Owned(strip_comment_remnants(&s1))
        } else {
            s1
        };
        let s2_trimmed = s2.trim();
        let s3: std::borrow::Cow<str> = if s2_trimmed.contains("  ") {
            // Collapse consecutive spaces by scanning bytes directly.
            std::borrow::Cow::Owned(collapse_spaces(s2_trimmed))
        } else {
            std::borrow::Cow::Borrowed(s2_trimmed)
        };
        let cleaned = s3.trim();

        if cleaned.is_empty() {
            blank_run += 1;
            continue;
        }

        // Insert separator before the content line.
        if !result.is_empty() {
            if blank_run >= 1 {
                // One or more blank lines → paragraph break (\n\n).
                result.push_str("\n\n");
            } else {
                result.push('\n');
            }
        }

        result.push_str(cleaned);
        blank_run = 0;
    }

    result
}

/// Cleans wikitext using regex-based heuristics as a fallback.
///
/// Used when the AST parser fails. Applies a series of regex substitutions
/// to remove or simplify common wikitext constructs.
///
/// # Arguments
///
/// * `wikitext` - The raw wikitext markup to clean.
///
/// # Returns
///
/// A `String` with markup removed via regex patterns.
fn fallback_clean(wikitext: &str) -> String {
    use std::borrow::Cow;

    // Keep the text as Cow<str> throughout.  replace_all returns Cow::Borrowed
    // when the regex finds no match, so no String clone is made unless the
    // pattern actually matches.  The `sub!` macro scopes the borrow of `text`
    // strictly inside the replace_all call; only the owned result is assigned.
    let mut text: Cow<str> = Cow::Borrowed(wikitext);

    macro_rules! sub {
        ($re:expr, $rep:expr) => {
            if let Cow::Owned(s) = $re.replace_all(text.as_ref(), $rep) {
                text = Cow::Owned(s);
            }
        };
    }

    // Remove ref tags first (before general HTML tag removal).
    sub!(RE_REF, "");

    // Remove tables.
    sub!(RE_TABLE, "");

    // Remove templates (iterate for nested templates).
    for _ in 0..10 {
        match RE_TEMPLATE.replace_all(text.as_ref(), "") {
            Cow::Owned(s) => text = Cow::Owned(s),
            Cow::Borrowed(_) => break,
        }
    }

    // Remove category and file links.
    sub!(RE_CATEGORY, "");
    sub!(RE_FILE, "");

    // Convert piped links to display text.
    sub!(RE_PIPED_LINK, "$1");

    // Convert simple links to target text.
    sub!(RE_SIMPLE_LINK, "$1");

    // Convert external links to label text.
    sub!(RE_EXTERNAL_LINK, "$1");

    // Remove bold/italic markup.
    sub!(RE_BOLD, "$1");
    sub!(RE_ITALIC, "$1");

    // Convert headings to plain text.
    sub!(RE_HEADING, "$1");

    // Remove HTML tags.
    sub!(RE_HTML_TAG, "");

    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_passthrough() {
        let input = "This is plain text.";
        let result = clean_wikitext(input);
        assert_eq!(result, "This is plain text.");
    }

    #[test]
    fn test_bold_italic_removal() {
        let input = "This is '''bold''' and ''italic'' text.";
        let result = clean_wikitext(input);
        assert_eq!(result, "This is bold and italic text.");
    }

    #[test]
    fn test_internal_link_without_pipe() {
        let input = "Visit [[Main Page]] for more.";
        let result = clean_wikitext(input);
        assert_eq!(result, "Visit Main Page for more.");
    }

    #[test]
    fn test_internal_link_with_pipe() {
        let input = "See [[United States|the US]] for details.";
        let result = clean_wikitext(input);
        assert_eq!(result, "See the US for details.");
    }

    #[test]
    fn test_external_link() {
        let input = "Visit [http://example.com Example Site] for more.";
        let result = clean_wikitext(input);
        assert!(
            result.contains("Example Site"),
            "Expected label text, got: {result}"
        );
    }

    #[test]
    fn test_template_removal() {
        let input = "Before {{cite web|url=http://example.com|title=Test}} after.";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("cite web"),
            "Template should be removed, got: {result}"
        );
        assert!(
            result.contains("Before"),
            "Text before template should remain, got: {result}"
        );
        assert!(
            result.contains("after."),
            "Text after template should remain, got: {result}"
        );
    }

    #[test]
    fn test_table_removal() {
        let input =
            "Before table.\n{| class=\"wikitable\"\n|-\n! Header\n|-\n| Cell\n|}\nAfter table.";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("Header"),
            "Table content should be removed, got: {result}"
        );
        assert!(
            result.contains("Before table."),
            "Text before table should remain, got: {result}"
        );
    }

    #[test]
    fn test_comment_removal() {
        let input = "Visible <!-- hidden comment --> text.";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("hidden comment"),
            "Comment should be removed, got: {result}"
        );
        assert!(
            result.contains("Visible"),
            "Visible text should remain, got: {result}"
        );
    }

    #[test]
    fn test_heading_extraction() {
        let input = "== Section Title ==\nContent here.";
        let result = clean_wikitext(input);
        assert!(
            result.contains("Section Title"),
            "Heading text should be extracted, got: {result}"
        );
        assert!(
            !result.contains("=="),
            "Heading markers should be removed, got: {result}"
        );
    }

    #[test]
    fn test_category_removal() {
        let input = "Text content.\n[[Category:Example]]";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("Category"),
            "Category should be removed, got: {result}"
        );
        assert!(
            result.contains("Text content."),
            "Regular text should remain, got: {result}"
        );
    }

    #[test]
    fn test_image_removal() {
        let input = "Text content.\n[[File:Example.jpg|thumb|Caption]]";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("Example.jpg"),
            "Image link should be removed, got: {result}"
        );
    }

    #[test]
    fn test_ref_tag_removal() {
        let input = "Fact<ref>Source: Book, 2024</ref> is stated.";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("Source"),
            "Ref content should be removed, got: {result}"
        );
        assert!(
            result.contains("Fact"),
            "Text before ref should remain, got: {result}"
        );
    }

    #[test]
    fn test_unordered_list() {
        let input = "* Item one\n* Item two\n* Item three";
        let result = clean_wikitext(input);
        assert!(
            result.contains("Item one"),
            "List items should be extracted, got: {result}"
        );
        assert!(
            result.contains("Item two"),
            "List items should be extracted, got: {result}"
        );
    }

    #[test]
    fn test_paragraph_break() {
        let input = "First paragraph.\n\nSecond paragraph.";
        let result = clean_wikitext(input);
        assert!(
            result.contains("First paragraph."),
            "First paragraph should be present, got: {result}"
        );
        assert!(
            result.contains("Second paragraph."),
            "Second paragraph should be present, got: {result}"
        );
    }

    #[test]
    fn test_complex_wikitext() {
        let input = concat!(
            "'''Albert Einstein''' was a [[Germany|German]]-born ",
            "[[theoretical physics|theoretical physicist]]",
            "{{cite web|url=http://example.com}} ",
            "who developed the [[theory of relativity]].",
            "<!-- draft note -->",
            "<ref>Physics Today, 2024</ref>",
        );
        let result = clean_wikitext(input);
        assert!(
            result.contains("Albert Einstein"),
            "Name should be present, got: {result}"
        );
        assert!(
            result.contains("German"),
            "Link display text should be present, got: {result}"
        );
        assert!(
            result.contains("theoretical physicist"),
            "Link display text should be present, got: {result}"
        );
        assert!(
            result.contains("theory of relativity"),
            "Simple link text should be present, got: {result}"
        );
        assert!(
            !result.contains("cite web"),
            "Template should be removed, got: {result}"
        );
        assert!(
            !result.contains("draft note"),
            "Comment should be removed, got: {result}"
        );
        assert!(
            !result.contains("Physics Today"),
            "Ref should be removed, got: {result}"
        );
    }

    #[test]
    fn test_fallback_cleaning() {
        let input = "'''Bold''' and ''italic'' with [[Link|display]] and [[Simple]] link.";
        let result = fallback_clean(input);
        assert!(
            result.contains("Bold"),
            "Bold text should remain, got: {result}"
        );
        assert!(
            result.contains("italic"),
            "Italic text should remain, got: {result}"
        );
        assert!(
            result.contains("display"),
            "Piped link display should remain, got: {result}"
        );
        assert!(
            result.contains("Simple"),
            "Simple link target should remain, got: {result}"
        );
        assert!(
            !result.contains("'''"),
            "Bold markers should be removed, got: {result}"
        );
        assert!(
            !result.contains("''"),
            "Italic markers should be removed, got: {result}"
        );
    }

    #[test]
    fn test_fallback_template_removal() {
        let input = "Before {{outer|{{inner}}}} after.";
        let result = fallback_clean(input);
        assert!(
            !result.contains("{{"),
            "Nested templates should be removed, got: {result}"
        );
    }

    #[test]
    fn test_fallback_ref_removal() {
        let input = "Text<ref name=\"a\">Citation</ref> and <ref/> end.";
        let result = fallback_clean(input);
        assert!(
            !result.contains("Citation"),
            "Ref content should be removed, got: {result}"
        );
    }

    #[test]
    fn test_clean_text_whitespace() {
        let input = "  Line one  \n\n\n\n  Line two  \n  Line three  ";
        let result = clean_text(input);
        assert_eq!(result, "Line one\n\nLine two\nLine three");
    }

    #[test]
    fn test_orphaned_template_close_removal() {
        let input = "amp;}}";
        let result = clean_text(input);
        assert_eq!(
            result, "",
            "Orphaned template close should be removed, got: {result}"
        );
    }

    #[test]
    fn test_template_param_line_removal() {
        let input = "本文テキスト。\n|image5 = Example.jpg|caption5 = 説明文}}\n続きのテキスト。";
        let result = clean_text(input);
        assert!(
            !result.contains("image5"),
            "Template parameter line should be removed, got: {result}"
        );
        assert!(
            result.contains("本文テキスト。"),
            "Body text should remain, got: {result}"
        );
        assert!(
            result.contains("続きのテキスト。"),
            "Following text should remain, got: {result}"
        );
    }

    #[test]
    fn test_comment_remnant_removal() {
        let input = "!--tlh:Hol--";
        let result = clean_text(input);
        assert_eq!(
            result, "",
            "Comment remnant should be removed, got: {result}"
        );
    }

    #[test]
    fn test_comment_remnant_inline() {
        let input = "Visible text !--hidden comment-- more text.";
        let result = clean_text(input);
        assert!(
            !result.contains("hidden comment"),
            "Inline comment remnant should be removed, got: {result}"
        );
        assert!(
            result.contains("Visible text"),
            "Text before comment should remain, got: {result}"
        );
        assert!(
            result.contains("more text."),
            "Text after comment should remain, got: {result}"
        );
    }

    #[test]
    fn test_japanese_category_removal() {
        let input = "本文テキスト。\n[[カテゴリ:日本の都市]]";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("カテゴリ"),
            "Japanese category should be removed, got: {result}"
        );
        assert!(
            result.contains("本文テキスト。"),
            "Body text should remain, got: {result}"
        );
    }

    #[test]
    fn test_japanese_file_removal() {
        let input = "本文テキスト。\n[[ファイル:Example.jpg|thumb|説明文]]";
        let result = clean_wikitext(input);
        assert!(
            !result.contains("Example.jpg"),
            "Japanese file link should be removed, got: {result}"
        );
        assert!(
            !result.contains("ファイル"),
            "Japanese file namespace should be removed, got: {result}"
        );
    }

    #[test]
    fn test_japanese_internal_link() {
        let input = "[[東京都|東京]]は日本の首都である。";
        let result = clean_wikitext(input);
        assert!(
            result.contains("東京"),
            "Japanese link display text should be present, got: {result}"
        );
        assert!(
            !result.contains("東京都"),
            "Japanese link target should be removed when pipe is used, got: {result}"
        );
    }

    #[test]
    fn test_japanese_simple_link() {
        let input = "[[大阪府]]は日本にある。";
        let result = clean_wikitext(input);
        assert!(
            result.contains("大阪府"),
            "Japanese simple link text should be present, got: {result}"
        );
        assert!(
            !result.contains("[["),
            "Link brackets should be removed, got: {result}"
        );
    }

    #[test]
    fn test_japanese_complex_wikitext() {
        let input = concat!(
            "'''日本語'''（にほんご、にっぽんご）は、",
            "[[日本]]国内や、かつての[[大日本帝国|日本統治地域]]で使われている",
            "[[言語]]である。",
            "{{lang|ja|日本語}}",
            "<!-- hidden -->",
            "<ref>出典情報</ref>",
            "\n[[カテゴリ:日本の言語]]",
            "\n[[ファイル:Japanese.png|thumb|日本語]]",
        );
        let result = clean_wikitext(input);
        assert!(
            result.contains("日本語"),
            "Title text should be present, got: {result}"
        );
        assert!(
            result.contains("にほんご"),
            "Furigana text should be present, got: {result}"
        );
        assert!(
            result.contains("日本統治地域"),
            "Piped link display text should be present, got: {result}"
        );
        assert!(
            result.contains("言語"),
            "Simple link text should be present, got: {result}"
        );
        assert!(
            !result.contains("hidden"),
            "Comment should be removed, got: {result}"
        );
        assert!(
            !result.contains("出典情報"),
            "Ref should be removed, got: {result}"
        );
        assert!(
            !result.contains("カテゴリ"),
            "Japanese category should be removed, got: {result}"
        );
        assert!(
            !result.contains("ファイル"),
            "Japanese file link should be removed, got: {result}"
        );
    }

    #[test]
    fn test_fallback_japanese_category_removal() {
        let input = "テキスト。\n[[カテゴリ:テスト]]";
        let result = fallback_clean(input);
        assert!(
            !result.contains("カテゴリ"),
            "Japanese category should be removed in fallback, got: {result}"
        );
    }

    #[test]
    fn test_fallback_japanese_file_removal() {
        let input = "テキスト。\n[[ファイル:Test.png|thumb|説明]]";
        let result = fallback_clean(input);
        assert!(
            !result.contains("ファイル"),
            "Japanese file should be removed in fallback, got: {result}"
        );
    }
}
