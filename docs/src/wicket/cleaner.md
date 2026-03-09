# cleaner

The `cleaner` module converts MediaWiki wikitext markup into plain text.

## Functions

### `clean_wikitext(wikitext: &str) -> String`

Converts raw wikitext into clean plain text by removing all MediaWiki markup.

The cleaning process uses a three-stage approach:

1. **AST-based cleaning** -- Uses `parse_wiki_text_2` to parse the wikitext into an AST and extracts text content from relevant nodes
2. **Regex fallback** -- When AST parsing fails or for markup not handled by the AST, applies regex-based pattern removal
3. **Post-processing** -- Removes markup remnants that survive the first two stages, such as orphaned template braces (`}}`), template parameter lines, and HTML comment fragments

The parser is configured with both English and Japanese Wikipedia namespaces,
so it correctly handles dumps from either language edition without requiring
any configuration changes.

### Handled Markup

The cleaner handles the following MediaWiki markup elements:

- **Bold/Italic** -- `'''bold'''` and `''italic''`
- **Internal links** -- `[[Article]]` and `[[Article|display text]]`
- **External links** -- `[https://example.com text]`
- **Templates** -- `{{template|...}}`
- **HTML tags** -- `<ref>`, `<nowiki>`, `<gallery>`, etc.
- **Categories** -- `[[Category:...]]` and `[[カテゴリ:...]]`
- **Files** -- `[[File:...]]`, `[[Image:...]]`, and `[[ファイル:...]]`
- **Tables** -- Wikitext table markup
- **Comments** -- `<!-- comments -->`
- **Magic words** -- `__TOC__`, `__NOTOC__`, etc.
- **Redirects** -- `#REDIRECT` and `#転送`

## Usage

```rust
use wicket::clean_wikitext;

let wikitext = "'''April''' is the [[month|fourth month]] of the year.";
let text = clean_wikitext(wikitext);
assert_eq!(text, "April is the fourth month of the year.");
```
