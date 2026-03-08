# extractor

`extractor` モジュールは抽出された記事を最終的な出力形式にフォーマットします。

## 型

### `OutputFormat`

出力フォーマットを指定する列挙型。

| バリアント | 説明 |
| ----- | ----- |
| `Doc` | wikiextractor 互換の XML 風タグ付き doc フォーマット |
| `Json` | JSON Lines フォーマット（1記事1JSON オブジェクト） |

## 関数

### `format_page(id: u64, title: &str, url: &str, text: &str, format: OutputFormat) -> String`

1つの記事を指定されたフォーマットで整形します。

**doc フォーマット出力:**

```xml
<doc id="1" url="https://en.wikipedia.org/wiki/April" title="April">
April is the fourth month of the year...
</doc>
```

**JSON フォーマット出力:**

```json
{"id":"1","url":"https://en.wikipedia.org/wiki/April","title":"April","text":"April is the fourth month of the year..."}
```

### `make_url(url_base: &str, title: &str) -> String`

URL ベースとタイトルから完全な Wikipedia 記事 URL を構築します。タイトル中のスペースはアンダースコアに置換されます。

### `parse_file_size(spec: &str) -> Result<u64, Error>`

人間が読めるファイルサイズ指定をバイト数に変換します。

| 入力 | 結果 |
| ----- | ----- |
| `"1M"` | 1,048,576 |
| `"500K"` | 512,000 |
| `"1G"` | 1,073,741,824 |
| `"0"` | 0（1記事1ファイル） |
