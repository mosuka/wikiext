# インストール

## 前提条件

- **Rust 1.85 以上**（stable チャンネル）-- [rust-lang.org](https://www.rust-lang.org/) から入手
- **Cargo**（Rust のパッケージマネージャ、Rust に同梱）

## CLI ツールのインストール

### crates.io から

```sh
cargo install wicket-cli
```

### ソースから

```sh
git clone https://github.com/mosuka/wext.git
cd wext
cargo build --release
```

バイナリは `./target/release/wicket` に生成されます。

インストールの確認:

```sh
./target/release/wicket --help
```

## ライブラリとして使用

プロジェクトの `Cargo.toml` に追加:

```toml
[dependencies]
wicket = "0.1.0"
```

## サポートプラットフォーム

wicket は以下のプラットフォームでテストされています:

| OS | アーキテクチャ |
| ----- | ----- |
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), aarch64 (Apple Silicon) |
| Windows | x86_64, aarch64 |
