---
title: "parser/tsrx"
description: "リンターが報告する TSRX モジュールの構文解析・変換の診断と、その修正方法を説明します。"
---

<!-- Reviewed translation; source: rules/reference/parser-tsrx.md -->

# `parser/tsrx`

`.tsrx` モジュールの構文解析や、マークアップ領域の中間表現への変換で問題が報告されると、リンターがこの診断を使います。
プリセットのルール一覧とは別に発生する処理上の診断です。自動修正はありません。

## 診断が出る条件

`.tsrx` モジュールは、ホストの TypeScript とマークアップをまとめて 1 回だけ解析します。続いて各マークアップ領域を
JSX・TSX の描画ルートと同じ中間表現に変換し、マークアップのルールがすべての領域を検査します。

- 構文エラーやテンプレート規則のエラーは、そのソース範囲に報告されます。TSRX の文法が出すメッセージは
  `TSRX1002` のような仕様のコードで始まります。最初の構文エラーで解析が止まるため、モジュールのマークアップは検査されません。
- `@switch` や `@try` など、変換処理がまだ対応していない構文は、その構文の位置に報告され、
  モジュールのマークアップは検査されません。メッセージに構文の種類が含まれます。

どちらもエラーです。

## 悪い例

`div` 要素を `</span>` で閉じています。

```tsx annotate="remove:2"
export function Notice() @{
  <div>Notice</span>
}
```

## 良い例

各要素を同じ名前の終了タグで閉じます。

```tsx annotate="add:2"
export function Notice() @{
  <div>Notice</div>
}
```

## 修正方法

報告された範囲の構文を修正して、lint を再実行します。要素名の対応、式を囲む波括弧、
ディレクティブのブロック、周囲の TypeScript を確認してください。

まだ対応していない構文を含むモジュールは、対応するまで検査されません。CI が止まる場合は `ignores` で
lint の対象から外し、メッセージ全体とともにその構文を報告してください。lint のプリセットを変えても、どちらの結果も変わりません。

## 関連ページ

- [CLI による lint](../../guide/cli.md#lint)
- [ネイティブの lint 実行設定](../../guide/lint-execution.md)
- [診断変換のソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/tsrx.rs)
- [TSRX の変換処理](https://github.com/ubugeeei-prod/vize/blob/main/davinci/vize_l1_to_l2/src/framework/vue/tsrx.rs)
