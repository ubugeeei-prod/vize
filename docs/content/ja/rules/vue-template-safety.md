---
title: "テンプレートの安全性と HTML の構造"
---

<span id="vue-rules-template-safety"></span>

# テンプレートの安全性と HTML の構造

このルール群では、HTML の挿入や URL の指定、属性と ID の重複、textarea・コンポーネントへのテキストの渡し方、HTML の入れ子を確認します。

外部から受け取った値を表示するときや、リンク・フォームを追加するとき、同じコンポーネントを複数表示するときに確認してください。

たとえば、[HTML を挿入せずテキストとして表示する例](./reference/vue-no-v-html.md)と[フォームの ID を生成して関連付ける例](./reference/vue-use-unique-element-ids.md)では、悪い例と良い例の差を確認できます。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。[Vue の全ルール](./vue.md)から関連するテンプレートの検査も確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-duplicate-attributes`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-duplicate-attributes) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-duplicate-attributes-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-duplicate-attributes-good) | 同じ要素の属性重複を検出します。 |
| [`vue/no-textarea-mustache`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-textarea-mustache) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-textarea-mustache-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-textarea-mustache-good) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |
| [`vue/no-unsafe-url`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-unsafe-url) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-unsafe-url-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-unsafe-url-good) | 危険なスキームを含む可能性がある URL 属性やバインディングを検出します。 |
| [`vue/no-v-html`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-html) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-html-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-html-good) | XSS を防ぐため、v-html を使用した箇所に警告を出します。 |
| [`vue/no-v-text-v-html-on-component`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component-good) | コンポーネントでの v-text / v-html を検出します。 |
| [`vue/permitted-contents`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-permitted-contents) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-permitted-contents-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-permitted-contents-good) | HTML の要素ごとのコンテンツモデルを検査します。 |
| [`vue/use-unique-element-ids`](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-use-unique-element-ids) | [悪い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-use-unique-element-ids-bad) · [良い](https://vizejs.dev/ja/rules/vue-template-safety.html#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で一意な ID を生成します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
