# ライブラリ API 概要

`wicket` クレートは、Wikipedia XML ダンプファイルからプレーンテキストを抽出する Rust API を提供します。

## インストール

```toml
[dependencies]
wicket = "0.1.0"
```

## モジュール一覧

| モジュール | 主な型 | 用途 |
| ----- | ----- | ----- |
| `wicket::dump` | `Article`, `DumpReader`, `open_dump()` | ストリーミング XML ダンプパース |
| `wicket::cleaner` | `clean_wikitext()` | Wikitext → プレーンテキスト変換 |
| `wicket::extractor` | `OutputFormat`, `format_page()`, `make_url()` | 出力フォーマット |
| `wicket::output` | `OutputConfig`, `OutputSplitter` | ファイル分割とローテーション |
| `wicket::error` | `Error` | エラー型定義 |

## 使用例

```rust
use wicket::{open_dump, clean_wikitext, format_page, make_url, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_dump("dump.xml.bz2".as_ref(), &[0])?;
    let url_base = reader.url_base().to_string();

    for result in reader {
        let article = result?;
        let text = clean_wikitext(&article.text);
        let url = make_url(&url_base, &article.title);
        let output = format_page(
            article.id, &article.title, &url, &text, OutputFormat::Doc,
        );
        print!("{}", output);
    }

    Ok(())
}
```

## API ドキュメント

完全な API ドキュメントは [docs.rs/wicket](https://docs.rs/wicket) で参照できます。
