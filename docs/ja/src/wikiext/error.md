# error

`error` モジュールは wikiext ライブラリ全体で使用されるエラー型を定義します。

## エラー型

`Error` 列挙型は `thiserror` を使用して導出され、すべてのエラー条件をカバーします:

| バリアント | ソース | 説明 |
| ----- | ----- | ----- |
| `Io` | `std::io::Error` | ファイル I/O エラー |
| `XmlReader` | `quick_xml::Error` | XML パースエラー |
| `JsonSerialization` | `serde_json::Error` | JSON シリアライゼーションエラー |

## Result 型

ライブラリは `Result` 型エイリアスを提供します:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

ライブラリのすべてのパブリック関数はこの `Result` 型を返します。
