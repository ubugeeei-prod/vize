---
title: "ディレクティブの構文と配置"
---

<span id="vue-rules-directive-validity"></span>

# ディレクティブの構文と配置

このルール群では、v-bind・v-if・v-for・v-model などのディレクティブに必要な値や引数、配置を確認します。

条件分岐や繰り返し、入力との連動、イベント処理、スロットを追加したときに確認してください。v-else の前後関係や、v-for の書式など、各ルールが検査する範囲を掲載しています。

まずは [v-for の繰り返し式と key の例](./reference/vue-valid-v-for.md)と[スロットの宣言の例](./reference/vue-valid-v-slot.md)で、悪い例・良い例の違いを確認してください。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行します。[Vue の全ルール](./vue.md)では、テンプレートの関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/valid-v-bind`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-bind) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-bind-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-bind-good) | v-bind の引数・値・修飾子を検査します。 |
| [`vue/valid-v-else`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-else) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-else-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-else-good) | v-else の位置・引数・値を検査します。 |
| [`vue/valid-v-for`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-for) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-for-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-for-good) | v-for の式と変数宣言を検査します。 |
| [`vue/valid-v-if`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-if) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-if-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-if-good) | v-if に有効な条件式を指定します。 |
| [`vue/valid-v-memo`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-memo) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-memo-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-memo-good) | v-memo の値を配列の式にします。 |
| [`vue/valid-v-model`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-model) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-model-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-model-good) | v-model の値・引数・修飾子を検査します。 |
| [`vue/valid-v-on`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-on) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-on-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-on-good) | v-on のイベント名・式・修飾子を検査します。 |
| [`vue/valid-v-show`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-show) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-show-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-show-good) | v-show に有効な条件式を指定します。 |
| [`vue/valid-v-slot`](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-slot) | [悪い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-slot-bad) · [良い](https://vizejs.dev/ja/rules/vue-directive-validity.html#vue-valid-v-slot-good) | v-slot の適用先・宣言・修飾子を検査します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
