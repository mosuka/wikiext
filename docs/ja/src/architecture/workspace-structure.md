# ワークスペース構成

wicket は **Cargo workspace** で2つのクレートと関連ディレクトリを管理しています。

## ディレクトリ構成

```text
wext/
├── Cargo.toml              # Workspace マニフェスト
├── Cargo.lock              # 依存ロックファイル
├── LICENSE                 # MIT OR Apache-2.0
├── README.md               # プロジェクト概要
├── wicket/                # コアライブラリクレート
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # モジュール宣言と再エクスポート
│       ├── dump.rs         # XML ダンプ ストリーミングパーサー
│       ├── cleaner.rs      # Wikitext → プレーンテキスト変換
│       ├── extractor.rs    # 出力フォーマット (doc/JSON)
│       ├── output.rs       # ファイル分割とローテーション
│       └── error.rs        # エラー型
├── wicket-cli/            # CLI バイナリクレート
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # CLI エントリーポイント
├── docs/                   # mdBook ドキュメント
│   ├── book.toml
│   ├── src/
│   └── ja/                 # 日本語ドキュメント
│       ├── book.toml
│       └── src/
└── .github/
    └── workflows/          # CI/CD パイプライン
        ├── regression.yml  # push/PR 時のテスト
        ├── release.yml     # リリースビルドと公開
        ├── periodic.yml    # 週次安定性テスト
        └── deploy-docs.yml # ドキュメントデプロイ
```

## クレート詳細

### `wicket` (コアライブラリ)

ストリーミング XML パース、Wikitext クリーニング、出力フォーマット、ファイル分割を提供するコアライブラリです。

| 依存クレート | バージョン | 用途 |
| ----- | ----- | ----- |
| `quick-xml` | 0.39 | ストリーミング XML パース |
| `parse-wiki-text-2` | 0.2 | Wikitext AST パース |
| `regex` | 1.12 | フォールバック Wikitext クリーニング |
| `bzip2` | 0.6 | bzip2 圧縮/展開 |
| `serde` | 1.0 | シリアライゼーション |
| `serde_json` | 1.0 | JSON 出力フォーマット |
| `rayon` | 1.11 | データ並列処理（CLI で使用） |
| `thiserror` | 2.0 | エラー型導出 |
| `log` | 0.4 | ログファサード |

### `wicket-cli` (CLI バイナリ)

wicket の機能をコマンドラインインターフェースで提供します。

| 依存クレート | バージョン | 用途 |
| ----- | ----- | ----- |
| `clap` | 4.5 | コマンドライン引数パース |
| `rayon` | 1.11 | 並列バッチ処理 |
| `bzip2` | 0.6 | 圧縮出力サポート |
| `env_logger` | 0.11 | ログ出力 |
| `anyhow` | 1.0 | バイナリのエラーハンドリング |
| `wicket` | 0.1 | コアライブラリ（workspace メンバー） |

## ワークスペース設定

Cargo resolver version 3（Rust Edition 2024）を使用:

```toml
[workspace]
resolver = "3"
members = ["wicket", "wicket-cli"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"
```

共有依存はワークスペースレベルの `[workspace.dependencies]` で定義し、各クレートでは `{ workspace = true }` で参照します。
