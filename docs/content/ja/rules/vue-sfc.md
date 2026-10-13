---
title: "SFC ブロックの構成"
---

<span id="vue-rules-sfc-blocks"></span>

# SFC ブロックの構成

`<template>`、`<script>`、`<style>` の配置やスタイルの範囲を確認し、SFC の構成をそろえるためのルールをまとめています。ブロックを追加したときや、外部ファイルの `src`、`lang`、カスタムブロックを見直すときに使います。

ブロック順の既定値では script と template のどちらを先にしてもよく、style はその後に置きます。`lang` に関する 3 件は規約の説明で、現在の SFC lint では固有の診断を出しません。各ルールの対応範囲を確認し、プロジェクトに必要なものを選んでください。

まずは [SFC ブロックの並び順の例](./reference/vue-sfc-element-order.md)で違いを確かめてください。[Vue ルール](./vue.md)で設定方針を確認し、採用するルールを設定して `vp run lint` を実行します。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-preprocessor-lang`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-preprocessor-lang) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-preprocessor-lang-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-preprocessor-lang-good) | CSS プリプロセッサーより標準の CSS を使う方針を示します。 |
| [`vue/no-script-non-standard-lang`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-script-non-standard-lang) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-script-non-standard-lang-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-script-non-standard-lang-good) | script の非標準の lang 指定を避ける方針を示します。 |
| [`vue/no-src-attribute`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-src-attribute) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-src-attribute-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-src-attribute-good) | SFC ブロックの外部 src 指定を検出します。 |
| [`vue/no-template-lang`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-template-lang) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-template-lang-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-no-template-lang-good) | template の lang 指定を避ける方針を示します。 |
| [`vue/require-scoped-style`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-require-scoped-style) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-require-scoped-style-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-require-scoped-style-good) | style に scoped を指定する方針を適用します。 |
| [`vue/sfc-element-order`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-sfc-element-order) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-sfc-element-order-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-sfc-element-order-good) | SFC のトップレベルブロックを設定した順に並べます。 |
| [`vue/single-style-block`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-single-style-block) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-single-style-block-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-single-style-block-good) | SFC の style を一つのブロックにまとめます。 |
| [`vue/warn-custom-block`](https://vizejs.dev/ja/rules/vue-sfc.html#vue-warn-custom-block) | [悪い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-warn-custom-block-bad) · [良い](https://vizejs.dev/ja/rules/vue-sfc.html#vue-warn-custom-block-good) | SFC のカスタムブロックを検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
