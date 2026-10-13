---
title: "ディレクティブとイベントの書き方"
---

<span id="vue-rules-directive-conventions"></span>

# ディレクティブとイベントの書き方

このルール群では、v-bind・v-on・v-slot の表記、イベント名、修飾キー付きのイベント処理、カスタムディレクティブの使用方針を確認します。
チームで表記を揃えたり、クリック操作やスロット、独自のディレクティブを追加したりするときに確認してください。

たとえば、[修飾キーによるクリックの重複を分ける例](./reference/vue-use-v-on-exact.md)では、Ctrl キーを押したときに通常のハンドラーも動く指定と、`.exact` を使って分ける指定を比較できます。
採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。
[Vue の全ルール](./vue.md)では、ディレクティブの構文やテンプレートの関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/scoped-event-names`](https://vizejs.dev/ja/rules/vue-directives.html#vue-scoped-event-names) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-scoped-event-names-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-scoped-event-names-good) | イベント名を context:event の形式に揃えます。 |
| [`vue/use-v-on-exact`](https://vizejs.dev/ja/rules/vue-directives.html#vue-use-v-on-exact) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-use-v-on-exact-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-use-v-on-exact-good) | 修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。 |
| [`vue/v-bind-style`](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-bind-style) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-bind-style-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-bind-style-good) | v-bind の表記形式を揃えます。 |
| [`vue/v-on-style`](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-on-style) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-on-style-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-on-style-good) | v-on の表記形式を揃えます。 |
| [`vue/v-slot-style`](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-slot-style) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-slot-style-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-v-slot-style-good) | v-slot の表記形式を揃えます。 |
| [`vue/warn-custom-directive`](https://vizejs.dev/ja/rules/vue-directives.html#vue-warn-custom-directive) | [悪い](https://vizejs.dev/ja/rules/vue-directives.html#vue-warn-custom-directive-bad) · [良い](https://vizejs.dev/ja/rules/vue-directives.html#vue-warn-custom-directive-good) | 登録が必要なカスタムディレクティブを検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
