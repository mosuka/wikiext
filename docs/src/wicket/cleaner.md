# cleaner

The `cleaner` module converts MediaWiki wikitext markup into plain text.

## Functions

### `clean_wikitext(wikitext: &str) -> String`

Converts raw wikitext into clean plain text by removing all MediaWiki markup.

The cleaning process uses a two-stage approach:

1. **AST-based cleaning** -- Uses `parse_wiki_text_2` to parse the wikitext into an AST and extracts text content from relevant nodes
2. **Regex fallback** -- When AST parsing fails or for markup not handled by the AST, applies regex-based pattern removal

### Handled Markup

The cleaner handles the following MediaWiki markup elements:

- **Bold/Italic** -- `'''bold'''` and `''italic''`
- **Internal links** -- `[[Article]]` and `[[Article|display text]]`
- **External links** -- `[https://example.com text]`
- **Templates** -- `{{template|...}}`
- **HTML tags** -- `<ref>`, `<nowiki>`, `<gallery>`, etc.
- **Categories** -- `[[Category:...]]`
- **Tables** -- Wikitext table markup
- **Comments** -- `<!-- comments -->`
- **Magic words** -- `__TOC__`, `__NOTOC__`, etc.

## Usage

```rust
use wicket::clean_wikitext;

let wikitext = "'''April''' is the [[month|fourth month]] of the year.";
let text = clean_wikitext(wikitext);
assert_eq!(text, "April is the fourth month of the year.");
```
