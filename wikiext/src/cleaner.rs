/// Wikitext cleaner module.
///
/// Converts MediaWiki markup (wikitext) into plain text by parsing the wikitext
/// into an AST using `parse_wiki_text_2` and extracting text content. Falls back
/// to regex-based cleaning when AST parsing fails.
use std::sync::LazyLock;

use log::warn;
use parse_wiki_text_2::{Configuration, Node};
use regex::Regex;

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

/// Matches `[[Category:...]]` links.
static RE_CATEGORY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[Category:[^\]]*\]\]").expect("invalid regex"));

/// Matches `[[File:...]]` and `[[Image:...]]` links.
static RE_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[(?:File|Image):[^\]]*\]\]").expect("invalid regex"));

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

/// Matches multiple consecutive blank lines.
static RE_MULTI_NEWLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").expect("invalid regex"));

/// Matches multiple consecutive spaces.
static RE_MULTI_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" {2,}").expect("invalid regex"));

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
    let config = Configuration::default();
    let result = config.parse(wikitext);

    match result {
        Ok(output) => {
            let mut text = String::new();
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

/// Post-processes extracted text by normalizing whitespace.
///
/// Performs the following cleanup steps:
/// - Trims leading and trailing whitespace from each line
/// - Collapses consecutive spaces into a single space
/// - Collapses three or more consecutive newlines into two (one blank line)
/// - Trims leading and trailing whitespace from the entire result
///
/// # Arguments
///
/// * `text` - The raw extracted text to clean up.
///
/// # Returns
///
/// A `String` with normalized whitespace.
fn clean_text(text: &str) -> String {
    // Trim each line and collapse spaces
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            RE_MULTI_SPACE.replace_all(trimmed, " ").to_string()
        })
        .collect();

    let joined = lines.join("\n");

    // Collapse multiple blank lines into one
    let result = RE_MULTI_NEWLINE.replace_all(&joined, "\n\n");

    result.trim().to_string()
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
    let mut text = wikitext.to_string();

    // Remove ref tags first (before general HTML tag removal)
    text = RE_REF.replace_all(&text, "").to_string();

    // Remove tables
    text = RE_TABLE.replace_all(&text, "").to_string();

    // Remove templates (iterate for nested templates)
    for _ in 0..10 {
        let replaced = RE_TEMPLATE.replace_all(&text, "").to_string();
        if replaced == text {
            break;
        }
        text = replaced;
    }

    // Remove category and file links
    text = RE_CATEGORY.replace_all(&text, "").to_string();
    text = RE_FILE.replace_all(&text, "").to_string();

    // Convert piped links to display text
    text = RE_PIPED_LINK.replace_all(&text, "$1").to_string();

    // Convert simple links to target text
    text = RE_SIMPLE_LINK.replace_all(&text, "$1").to_string();

    // Convert external links to label text
    text = RE_EXTERNAL_LINK.replace_all(&text, "$1").to_string();

    // Remove bold/italic markup
    text = RE_BOLD.replace_all(&text, "$1").to_string();
    text = RE_ITALIC.replace_all(&text, "$1").to_string();

    // Convert headings to plain text
    text = RE_HEADING.replace_all(&text, "$1").to_string();

    // Remove HTML tags
    text = RE_HTML_TAG.replace_all(&text, "").to_string();

    text
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
        let input = "Before table.\n{| class=\"wikitable\"\n|-\n! Header\n|-\n| Cell\n|}\nAfter table.";
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
}
