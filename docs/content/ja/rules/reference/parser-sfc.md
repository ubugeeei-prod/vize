---
title: "parser/sfc"
description: "リンターが報告する単一ファイルコンポーネントの構造エラーと、その修正方法を説明します。"
---

<!-- Reviewed translation; source: rules/reference/parser-sfc.md -->

# `parser/sfc`

Vue の単一ファイルコンポーネントの構造を解析できないとき、リンターがこのエラーを報告します。
プリセットのルール一覧とは別に発生する処理上の診断です。自動修正はありません。

## 診断が出る条件

SFC パーサーは、各ブロックの内容を検査する前に、最上位のブロックを完全に識別する必要があります。
たとえば、最上位の `template` ブロックに終了タグがないと `parser/sfc` が出ます。
診断には SFC パーサーのメッセージと、元のソース内の位置が使われます。
解析に失敗した場合、後続の検査へ有効な SFC 記述子を渡せません。

Art の variant 処理でも、variant を元の SFC ソース内に位置付けられないと、このエラーを使います。
その場合はファイルの先頭に長さゼロの位置を示します。構文が正しいソースで再現する場合は、
`.art.vue` ファイルを保存して不具合を報告してください。ルールを抑制しても、
失われたソース間の対応は復元されません。

## 悪い例

内側の要素は完成していますが、外側の `template` ブロックを閉じていません。

```vue
<template>
  <div>Notice</div>
```

## 良い例

外側のブロックを閉じ、SFC パーサーがテンプレート全体を識別できるようにします。

```vue
<template>
  <div>Notice</div>
</template>
```

## 修正方法

メッセージに示された最上位のブロックを確認し、必要な終了タグを復元します。
引用符の欠落や編集中の不完全な構文によって、後続のソースがブロックの一部として
解釈されていないかも確認してください。SFC の構造を直してから lint を再実行します。

SFC のブロックが完成していて、テンプレート内部に問題がある場合は、
[`parser/template`](./parser-template.md) を参照してください。lint のプリセットを変えても、
SFC の構造は修復されません。

## 関連ページ

- [ネイティブの lint 実行設定](../../guide/lint-execution.md)
- [`parser/template`](./parser-template.md)
- [SFC 診断のソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/parse_diagnostics.rs)
- [Art variant 診断のソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/sfc/art.rs)
