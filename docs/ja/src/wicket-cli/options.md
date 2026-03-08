# CLI オプション

## 入力

```sh
wicket <INPUT>
```

入力ファイルは位置引数です。Wikipedia XML ダンプファイルで、非圧縮（`.xml`）または bzip2 圧縮（`.xml.bz2`）のいずれかです。圧縮はファイル拡張子で自動検出されます。

## 出力ディレクトリ

```sh
wicket dump.xml.bz2 -o output/
wicket dump.xml.bz2 -o -
```

`-o, --output <PATH>` -- 出力ディレクトリを指定します。デフォルトは `text`。

- ディレクトリパスを指定した場合、wikiextractor の命名規則（AA/wiki_00 など）でファイルを作成
- `-` を指定した場合、ファイル分割せずにすべての出力を stdout に書き込み

## ファイルサイズ

```sh
wicket dump.xml.bz2 -b 500K
wicket dump.xml.bz2 -b 1M
wicket dump.xml.bz2 -b 1G
wicket dump.xml.bz2 -b 0
```

`-b, --bytes <SIZE>` -- 出力ファイルの最大バイト数。デフォルトは `1M`。

サポートされるサフィックス: `K`（キロバイト）、`M`（メガバイト）、`G`（ギガバイト）。`0` を指定すると各記事が個別のファイルに書き込まれます。

## 圧縮

```sh
wicket dump.xml.bz2 -c
```

`-c, --compress` -- bzip2 で出力ファイルを圧縮。出力ファイルには `.bz2` 拡張子が付きます。

## JSON 出力

```sh
wicket dump.xml.bz2 --json
```

`--json` -- デフォルトの doc フォーマットの代わりに JSON Lines フォーマット（1行1JSON オブジェクト）で出力します。

## 並列ワーカー数

```sh
wicket dump.xml.bz2 --processes 8
```

`--processes <N>` -- テキストクリーニングの並列ワーカー数。デフォルトは CPU コア数。

## 静粛モード

```sh
wicket dump.xml.bz2 -q
```

`-q, --quiet` -- stderr への進捗出力を抑制。パイプで他のコマンドに出力を渡す際に便利です。

## 名前空間フィルタリング

```sh
wicket dump.xml.bz2 --namespaces 0
wicket dump.xml.bz2 --namespaces 0,1,2
```

`--namespaces <IDS>` -- 抽出する名前空間 ID のカンマ区切りリスト。デフォルトは `0`（メイン記事のみ）。

主な名前空間 ID:

| ID | 名前空間 |
| ----- | ----- |
| 0 | メイン（記事） |
| 1 | トーク |
| 2 | ユーザー |
| 3 | ユーザートーク |
| 4 | Wikipedia |
| 6 | ファイル |
| 10 | テンプレート |
| 14 | カテゴリ |
