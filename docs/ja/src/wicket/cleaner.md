# cleaner

`cleaner` モジュールは MediaWiki の Wikitext マークアップをプレーンテキストに変換します。

## 関数

### `clean_wikitext(wikitext: &str) -> String`

生の Wikitext を、すべての MediaWiki マークアップを除去したクリーンなプレーンテキストに変換します。

クリーニング処理は3段階のアプローチを使用します:

1. **AST ベースクリーニング** -- `parse_wiki_text_2` で Wikitext を AST にパースし、関連ノードからテキストコンテンツを抽出
2. **正規表現フォールバック** -- AST パース失敗時や AST で処理できないマークアップに対して、正規表現ベースのパターン除去を適用
3. **後処理** -- 最初の2段階で残ったマークアップ残骸（孤立したテンプレート閉じ括弧 `}}`、テンプレートパラメータ行、HTML コメント断片など）を除去

パーサーは英語版・日本語版両方の Wikipedia 名前空間に対応しており、
設定変更なしでどちらの言語のダンプも正しく処理できます。

### 対応するマークアップ

クリーナーは以下の MediaWiki マークアップ要素を処理します:

- **太字/斜体** -- `'''太字'''` および `''斜体''`
- **内部リンク** -- `[[記事]]` および `[[記事|表示テキスト]]`
- **外部リンク** -- `[https://example.com テキスト]`
- **テンプレート** -- `{{テンプレート|...}}`
- **HTML タグ** -- `<ref>`、`<nowiki>`、`<gallery>` など
- **カテゴリ** -- `[[Category:...]]` および `[[カテゴリ:...]]`
- **ファイル** -- `[[File:...]]`、`[[Image:...]]`、`[[ファイル:...]]`
- **テーブル** -- Wikitext テーブルマークアップ
- **コメント** -- `<!-- コメント -->`
- **マジックワード** -- `__TOC__`、`__NOTOC__` など
- **リダイレクト** -- `#REDIRECT` および `#転送`

## 使用例

```rust
use wicket::clean_wikitext;

let wikitext = "'''April''' is the [[month|fourth month]] of the year.";
let text = clean_wikitext(wikitext);
assert_eq!(text, "April is the fourth month of the year.");
```
