# error

The `error` module defines the error types used throughout the wicket library.

## Error Type

The `Error` enum is derived using `thiserror` and covers all error conditions:

| Variant | Source | Description |
| ----- | ----- | ----- |
| `Io` | `std::io::Error` | File I/O errors |
| `XmlReader` | `quick_xml::Error` | XML parsing errors |
| `JsonSerialization` | `serde_json::Error` | JSON serialization errors |

## Result Type

The library provides a `Result` type alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

All public functions in the library return this `Result` type.
