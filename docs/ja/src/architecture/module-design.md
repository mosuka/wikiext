# モジュール設計

`wikiext` ライブラリクレートは5つのモジュールで構成され、それぞれ明確な責務を持ちます。

## モジュール概要

| モジュール | 主な型 | 用途 |
| ----- | ----- | ----- |
| `dump` | `Article`, `DumpReader` | ストリーミング XML ダンプパース |
| `cleaner` | `clean_wikitext()` | Wikitext → プレーンテキスト変換 |
| `extractor` | `OutputFormat`, `format_page()` | 出力フォーマット (doc/JSON) |
| `output` | `OutputConfig`, `OutputSplitter` | ファイル分割とローテーション |
| `error` | `Error` | エラー型定義 |

## モジュール詳細

### `dump` -- XML ダンプリーダー

`quick-xml` を使用したストリーミング XML パーサー。MediaWiki XML ダンプファイルを読み込み、`Article` 構造体を生成します。

- **`Article`** -- `id` (u64)、`title` (String)、`namespace` (i32)、`text` (String) を持つ Wikipedia ページ
- **`DumpReader<R: BufRead>`** -- 名前空間フィルタリング付きで XML ソースから記事をストリーミングするイテレータ
- **`open_dump(path, namespaces)`** -- `MultiBzDecoder` による `.bz2` 自動検出付きでダンプファイルを開く

リーダーは `<siteinfo><base>` を解析して Wiki の URL ベースを抽出し、`url_base()` で公開します。

### `cleaner` -- Wikitext クリーナー

2段階のアプローチで MediaWiki マークアップをプレーンテキストに変換:

1. **AST ベースクリーニング** -- `parse_wiki_text_2` で Wikitext を AST にパースし、テキストコンテンツを抽出
2. **正規表現フォールバック** -- AST パース失敗時や AST で処理できないマークアップに対して正規表現ベースのクリーンアップを適用

主な関数: `clean_wikitext(wikitext: &str) -> String`

### `extractor` -- 出力フォーマッター

抽出された記事を最終的な出力形式にフォーマットします。

- **`OutputFormat`** -- `Doc` と `Json` のバリアントを持つ列挙型
- **`format_page(id, title, url, text, format)`** -- 1記事をフォーマット
- **`make_url(url_base, title)`** -- Wikipedia 記事の URL を構築
- **`parse_file_size(spec)`** -- `1M`、`500K`、`1G` などのサイズ指定をパース

### `output` -- ファイルスプリッター

wikiextractor のディレクトリ命名規則に従い、抽出記事を分割ファイルに書き込みます。

- **`OutputConfig`** -- 出力パス、最大ファイルサイズ、圧縮の設定
- **`OutputSplitter`** -- AA/wiki_00 命名によるファイルローテーション管理（1ディレクトリ100ファイル、AA〜ZZ）

stdout 出力（path = `"-"`）、bzip2 圧縮、ファイルサイズ制限をサポートします。

### `error` -- エラー型

`thiserror` を使用した `Error` 列挙型:

- `Io` -- I/O エラー
- `XmlReader` -- `quick-xml` の XML パースエラー
- `JsonSerialization` -- JSON シリアライゼーションエラー

## パブリックエクスポート

`lib.rs` では主要な型を利便性のために再エクスポートしています:

```rust
pub use cleaner::clean_wikitext;
pub use dump::{open_dump, Article, DumpReader};
pub use error::Error;
pub use extractor::{format_page, make_url, parse_file_size, OutputFormat};
pub use output::{OutputConfig, OutputSplitter};
```
