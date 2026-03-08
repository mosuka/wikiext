# CLI 使用例

## 基本的な抽出

Wikipedia ダンプからテキストをデフォルトの `text/` ディレクトリに抽出:

```sh
wikiext simplewiki-latest-pages-articles.xml.bz2
```

## カスタム出力ディレクトリ

```sh
wikiext dump.xml.bz2 -o output/
```

## 標準出力に書き出し

パイプで他のコマンドに直接出力:

```sh
wikiext dump.xml.bz2 -o - -q | wc -l
```

## JSON 出力 + 圧縮

```sh
wikiext dump.xml.bz2 -o output/ --json -c
```

## トークページの抽出

名前空間 1（トークページ）を 8 ワーカーで抽出:

```sh
wikiext dump.xml.bz2 -o output/ --namespaces 1 --processes 8
```

## 複数の名前空間

メイン記事とユーザーページを抽出:

```sh
wikiext dump.xml.bz2 -o output/ --namespaces 0,2
```

## 小さいファイルに分割

出力を 500 KB ファイルに分割:

```sh
wikiext dump.xml.bz2 -o output/ -b 500K
```

## 1記事1ファイル

```sh
wikiext dump.xml.bz2 -o output/ -b 0
```

## 出力ディレクトリ構造

抽出後の出力ディレクトリ:

```text
output/
  AA/
    wiki_00
    wiki_01
    ...
    wiki_99
  AB/
    wiki_00
    ...
```

`--compress` 使用時:

```text
output/
  AA/
    wiki_00.bz2
    wiki_01.bz2
    ...
```
