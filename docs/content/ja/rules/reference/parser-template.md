---
title: "parser/template"
description: "リンターが報告するテンプレートの構文診断と、その修正方法を説明します。"
---

<!-- Reviewed translation; source: rules/reference/parser-template.md -->

# `parser/template`

テンプレートのパーサーが不正な構文、ディレクティブや補間の不正な式、または style 内の
不正な `v-bind()` 式を見つけると、リンターがこの診断を報告します。
プリセットのルール一覧とは別に発生する処理上の診断です。自動修正はありません。

## 診断が出る条件

パーサーのメッセージとソース上の位置をそのまま使います。Vue SFC では、テンプレートの
先頭を起点に数え直さず、元の `.vue` ファイル内の位置を示します。
`&amp` や `&unknown;` のような不正な名前付き文字参照も対象です。
パーサーが回復可能と分類した構文エラーは警告、それ以外はエラーになります。
標準モードで HTML の自己終了構文を書き換えたことを知らせる互換性の通知は、lint の診断に含めません。

構文の回復と重大度は別の判断です。パーサーに回復処理が定義されている場合は、
エラーを報告しながら、回復後のツリーを意味解析のルールで検査できます。
回復処理が定義されていないエラーでは、不完全なツリーを使う意味解析を行いません。

## 悪い例

親の `div` が閉じる前に、`span` を閉じていません。

```vue
<template><div><span>Notice</div></template>
```

## 良い例

親要素を閉じる前に、内側の要素を閉じます。

```vue
<template>
  <div><span>Notice</span></div>
</template>
```

## 修正方法

報告された位置のメッセージを読み、周辺のタグ、属性名、引用符、文字参照、補間の区切り、
JavaScript 式を確認します。
ソースを修正してから lint を再実行してください。最初の構文エラーを直すと、
パーサーの回復処理に伴って出ていた後続の診断も解消する場合があります。

lint のプリセットを変えても、不正なテンプレートは修復されません。
SFC の外側のブロックに関する問題は、[`parser/sfc`](./parser-sfc.md) を参照してください。

## 関連ページ

- [ネイティブの lint 実行設定](../../guide/lint-execution.md)
- [`parser/sfc`](./parser-sfc.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/parse_diagnostics.rs)
