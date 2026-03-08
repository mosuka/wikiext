# dump

`dump` モジュールは MediaWiki ダンプファイルのストリーミング XML パースを提供します。

## 型

### `Article`

ダンプから抽出された1つの Wikipedia ページ。

| フィールド | 型 | 説明 |
| ----- | ----- | ----- |
| `id` | `u64` | ページ ID |
| `title` | `String` | ページタイトル |
| `namespace` | `i32` | 名前空間 ID（0 = メイン記事） |
| `text` | `String` | 生の Wikitext コンテンツ |

### `DumpReader<R: BufRead>`

XML ダンプソースから `Article` 値をストリーミングするイテレータ。

- `Iterator<Item = Result<Article, Error>>` を実装
- イテレータレベルで名前空間フィルタリング
- `url_base()` で `<siteinfo>` から取得した Wiki のベース URL を公開

## 関数

### `open_dump(path: &Path, namespaces: &[i32]) -> Result<DumpReader<...>>`

Wikipedia XML ダンプファイルを読み込み用に開きます。

- `.bz2` 拡張子を自動検出して `MultiBzDecoder` を適用
- `<siteinfo>` を解析して URL ベースを抽出
- 名前空間フィルタリングを設定

## 使用例

```rust
use wikiext::open_dump;

let reader = open_dump("dump.xml.bz2".as_ref(), &[0])?;
println!("URL base: {}", reader.url_base());

for result in reader {
    let article = result?;
    println!("[{}] {}", article.id, article.title);
}
```
