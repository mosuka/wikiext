# アーキテクチャ概要

wicket は高性能なストリーミング Wikipedia ダンプテキスト抽出ツールとして設計されています。ストリーミング I/O とバッチベースの並列処理を組み合わせて、数十 GB の XML ダンプを処理します。

## 全体のデータフロー

```text
入力 (.xml / .xml.bz2)
  |
  v
DumpReader (ストリーミング XML パース + 名前空間フィルタ)
  |  Article { id, title, namespace, text } を生成
  v
バッチ (1000 記事)
  |
  v
rayon par_iter (並列処理)
  |  clean_wikitext(text) -> プレーンテキスト
  |  format_page(id, title, url_base, text, format) -> フォーマット済み文字列
  v
OutputSplitter (逐次書き込み、ファイルローテーション)
  |
  v
出力ファイル (AA/wiki_00, AA/wiki_01, ...)
```

## 設計原則

- **ストリーミング処理** -- XML はストリームとしてパースされ、一度にメモリ上に置くのは1記事のみ
- **バッチ並列処理** -- CPU バウンドな Wikitext クリーニングを rayon で並列化し、I/O は逐次実行
- **wikiextractor 互換** -- 出力フォーマットとディレクトリ構造は Python 版オリジナルと一致
- **フェイルソフト** -- 不正なページはログに記録してスキップし、全体の処理は停止しない
- **ライブラリファースト** -- コア機能は `wicket` ライブラリクレートに集約。CLI は薄いラッパー
