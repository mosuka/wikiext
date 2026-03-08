# はじめに

**wikiext** は Wikipedia XML ダンプファイルからプレーンテキストを抽出する高性能ツールです。Python 製の [wikiextractor](https://github.com/attardi/wikiextractor) を Rust で再実装し、並列処理と効率的なストリーミングにより大幅な高速化を実現しています。

## 主な機能

- **ストリーミング XML パース** -- メモリに全体を読み込まず、数十 GB のダンプに対応
- **並列テキスト抽出** -- [rayon](https://crates.io/crates/rayon) による複数 CPU コアの活用
- **bzip2 自動展開** -- `.xml.bz2` ダンプファイルの透過的な展開
- **wikiextractor 互換出力** -- doc フォーマットおよび JSON フォーマット
- **ファイル分割** -- 出力ファイルの最大サイズを指定可能
- **名前空間フィルタリング** -- 特定のページ種別のみ抽出（メイン記事、トークページなど）

## 出力フォーマット

### doc フォーマット（デフォルト）

```xml
<doc id="1" url="https://en.wikipedia.org/wiki/April" title="April">
April is the fourth month of the year...
</doc>
```

### JSON フォーマット

```json
{"id":"1","url":"https://en.wikipedia.org/wiki/April","title":"April","text":"April is the fourth month of the year..."}
```

## 現在のバージョン

wikiext v0.1.0 -- Rust Edition 2024、最小 Rust バージョン 1.85。

## リンク

- [GitHub リポジトリ](https://github.com/mosuka/wext)
- [crates.io](https://crates.io/crates/wikiext)
- [API ドキュメント (docs.rs)](https://docs.rs/wikiext)
- [English Documentation](../../)
