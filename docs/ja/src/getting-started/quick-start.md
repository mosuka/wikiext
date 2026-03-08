# クイックスタート

## Wikipedia ダンプの入手

<https://dumps.wikimedia.org/> から Wikipedia ダンプをダウンロードします。テスト用には、サイズの小さい Simple English Wikipedia ダンプがおすすめです:

```sh
wget https://dumps.wikimedia.org/simplewiki/latest/simplewiki-latest-pages-articles.xml.bz2
```

## CLI クイックスタート

### 基本的な抽出

Wikipedia ダンプからプレーンテキストを抽出:

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o output/
```

ダンプを読み込み、メイン名前空間の全記事からプレーンテキストを抽出し、doc フォーマットで `output/` ディレクトリに出力します。ファイルは 1 MB ごとに分割されます。

### JSON 出力

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o output/ --json
```

### 標準出力に書き出し

```sh
wicket simplewiki-latest-pages-articles.xml.bz2 -o - -q | head -50
```

## ライブラリ クイックスタート

ダンプを開いて記事を処理する最小限の Rust プログラム:

```rust
use wicket::{open_dump, clean_wikitext, format_page, make_url, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = open_dump("simplewiki-latest-pages-articles.xml.bz2".as_ref(), &[0])?;
    let url_base = reader.url_base().to_string();

    for result in reader.take(5) {
        let article = result?;
        let text = clean_wikitext(&article.text);
        let url = make_url(&url_base, &article.title);
        let output = format_page(
            article.id, &article.title, &url, &text, OutputFormat::Doc,
        );
        println!("{}", output);
    }

    Ok(())
}
```

## 次のステップ

- [CLI リファレンス](../wicket-cli.md) -- 全 CLI オプション
- [アーキテクチャ](../architecture/overview.md) -- wicket の内部構造
