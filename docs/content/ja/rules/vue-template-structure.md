---
title: "繰り返し・条件分岐とテンプレートの構造"
---

<span id="vue-rules-template-structure"></span>

# 繰り返し・条件分岐とテンプレートの構造

繰り返しや条件分岐、スロットの変数と、template 要素の使い方を確認するルールをまとめています。リストの表示や分岐を変更したとき、テンプレートの構造を整理したときに使います。

リストでは項目を識別する key、条件分岐では重複する条件、入れ子のループやスロットでは変数の参照を確認します。no-template-shadow はテンプレート内の v-for・v-slot・従来のスコープ付きスロットの束縛を比較します。兄弟要素のスコープは独立しており、script 側と同じ名前であるだけでは診断しません。各ルールの現在の検査範囲を確認してください。

まずは [繰り返す要素に key を指定する例](./reference/vue-require-v-for-key.md)と[v-if と v-for を分ける例](./reference/vue-no-use-v-if-with-v-for.md)で、悪い例と良い例の差を確認してください。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行します。[Vue の全ルール](./vue.md)から、構文や属性の関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-child-content`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-child-content) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-child-content-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-child-content-good) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |
| [`vue/no-dupe-v-else-if`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-dupe-v-else-if) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-dupe-v-else-if-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-dupe-v-else-if-good) | v-if / v-else-if の条件重複を検出します。 |
| [`vue/no-lone-template`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-lone-template) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-lone-template-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-lone-template-good) | 不要な template 要素を検出します。 |
| [`vue/no-template-key`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-key) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-key-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-key-good) | v-for 用ではない template の key 指定を検出します。 |
| [`vue/no-template-shadow`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-shadow) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-shadow-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-template-shadow-good) | テンプレート変数が外側の名前を隠す箇所を検出します。 |
| [`vue/no-unused-vars`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-unused-vars) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-unused-vars-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-unused-vars-good) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |
| [`vue/no-use-v-if-with-v-for`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for-good) | 同じ要素での v-if と v-for の併用を検出します。 |
| [`vue/no-useless-template-attributes`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-useless-template-attributes) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-useless-template-attributes-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-no-useless-template-attributes-good) | template 要素の効果がない属性を検出します。 |
| [`vue/require-v-for-key`](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-require-v-for-key) | [悪い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-require-v-for-key-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-structure.html#vue-require-v-for-key-good) | v-for に安定した :key を指定します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
