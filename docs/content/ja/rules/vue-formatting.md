---
title: "テンプレートの属性と表記を揃える"
---

<span id="vue-rules-template-formatting"></span>

# テンプレートの属性と表記を揃える

このルール群では、属性名と並び順、引用符、空白、空の要素の書き方など、テンプレートの表記を確認します。チームで記述を揃えるときや、既存のテンプレートを整理するときに使ってください。

まずは [属性を並べる例](./reference/vue-attribute-order.md)と[同名の props を省略して渡す例](./reference/vue-prefer-props-shorthand.md)で、変更前後の違いを確認してください。同名の prop バインディングを省略する構文は Vue 3.4 以降が対象です。

採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。要素の種類ごとの自己終了タグの方針は[ルールのオプション](./options-vue.md)で指定できます。[Vue の全ルール](./vue.md)では、ディレクティブの構文やテンプレートの構造に関する検査も確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/attribute-hyphenation`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-hyphenation) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-hyphenation-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-hyphenation-good) | コンポーネントの prop 属性名を設定した形式に揃えます。 |
| [`vue/attribute-order`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-order) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-order-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-attribute-order-good) | テンプレートの属性を一定の順に並べます。 |
| [`vue/html-quotes`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-quotes) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-quotes-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-quotes-good) | HTML 属性値の引用符を揃えます。 |
| [`vue/html-self-closing`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-self-closing) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-self-closing-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-html-self-closing-good) | 要素の種類ごとに自己終了タグの形式を揃えます。 |
| [`vue/mustache-interpolation-spacing`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-mustache-interpolation-spacing) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-mustache-interpolation-spacing-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-mustache-interpolation-spacing-good) | mustache 内の空白を揃えます。 |
| [`vue/no-boolean-attr-value`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-boolean-attr-value) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-boolean-attr-value-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-boolean-attr-value-good) | HTML の真偽属性に不要な値を指定した箇所を検出します。 |
| [`vue/no-inline-style`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-inline-style) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-inline-style-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-inline-style-good) | インラインの style 属性を検出します。 |
| [`vue/no-multi-spaces`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-multi-spaces) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-multi-spaces-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-no-multi-spaces-good) | 連続する不要な空白を検出します。 |
| [`vue/prefer-props-shorthand`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prefer-props-shorthand) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prefer-props-shorthand-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prefer-props-shorthand-good) | Vue 3.4 の同名 prop バインディングの省略形を使います。 |
| [`vue/prop-name-casing`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prop-name-casing) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prop-name-casing-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-prop-name-casing-good) | 宣言する prop 名の形式を揃えます。 |
| [`vue/valid-attribute-name`](https://vizejs.dev/ja/rules/vue-formatting.html#vue-valid-attribute-name) | [悪い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-valid-attribute-name-bad) · [良い](https://vizejs.dev/ja/rules/vue-formatting.html#vue-valid-attribute-name-good) | 有効な属性名を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
