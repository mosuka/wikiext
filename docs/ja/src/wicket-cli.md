# CLI リファレンス概要

`wicket` CLI は Wikipedia XML ダンプファイルからプレーンテキストを抽出します。

## 使い方

```sh
wicket [OPTIONS] <INPUT>
```

## クイックリファレンス

| オプション | 説明 | デフォルト |
| ----- | ----- | ----- |
| `<INPUT>` | 入力 Wikipedia XML ダンプファイル（`.xml` または `.xml.bz2`） | (必須) |
| `-o, --output` | 出力ディレクトリ、または `-` で stdout | `text` |
| `-b, --bytes` | 出力ファイルの最大サイズ（例: `1M`, `500K`, `1G`） | `1M` |
| `-c, --compress` | bzip2 で出力ファイルを圧縮 | `false` |
| `--json` | JSON フォーマットで出力 | `false` |
| `--processes` | 並列ワーカー数 | CPU コア数 |
| `-q, --quiet` | stderr への進捗表示を抑制 | `false` |
| `--namespaces` | 抽出対象の名前空間 ID（カンマ区切り） | `0` |

## 詳細ドキュメント

- [オプション](wicket-cli/options.md) -- 全 CLI オプションの詳細説明
- [使用例](wicket-cli/examples.md) -- よくある使用パターンと例
