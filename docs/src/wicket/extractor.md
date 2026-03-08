# extractor

The `extractor` module formats extracted articles into the final output representation.

## Types

### `OutputFormat`

An enum specifying the output format.

| Variant | Description |
| ----- | ----- |
| `Doc` | wikiextractor-compatible doc format with XML-like tags |
| `Json` | JSON Lines format (one JSON object per article) |

## Functions

### `format_page(id: u64, title: &str, url: &str, text: &str, format: OutputFormat) -> String`

Formats a single article into the specified output format.

**Doc format output:**

```xml
<doc id="1" url="https://en.wikipedia.org/wiki/April" title="April">
April is the fourth month of the year...
</doc>
```

**JSON format output:**

```json
{"id":"1","url":"https://en.wikipedia.org/wiki/April","title":"April","text":"April is the fourth month of the year..."}
```

### `make_url(url_base: &str, title: &str) -> String`

Constructs a full Wikipedia article URL from the URL base and title. Spaces in the title are replaced with underscores.

### `parse_file_size(spec: &str) -> Result<u64, Error>`

Parses a human-readable file size specification into bytes.

| Input | Result |
| ----- | ----- |
| `"1M"` | 1,048,576 |
| `"500K"` | 512,000 |
| `"1G"` | 1,073,741,824 |
| `"0"` | 0 (one article per file) |
