---
title: "コンポーネントと props のルール"
---

<span id="vue-rules-components-and-props"></span>

# コンポーネントと props のルール

このルール群では、コンポーネントの名前・登録・使い方と、props の変更や未使用の宣言を確認します。
コンポーネントを追加したり、親子間で渡すデータや動的コンポーネントの切り替えを変更したりするときに確認してください。

たとえば、[props を直接変更しない例](./reference/vue-no-mutating-props.md)では、親から受け取った値を書き換える代わりに、更新をイベントで親へ通知する形を比較できます。
掲載された設定でルールを有効にし、`vp run lint` を実行してください。
[Vue の全ルール](./vue.md)では、テンプレートや SFC の関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/component-definition-name-casing`](https://vizejs.dev/ja/rules/vue-components.html#vue-component-definition-name-casing) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-component-definition-name-casing-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-component-definition-name-casing-good) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |
| [`vue/component-name-in-template-casing`](https://vizejs.dev/ja/rules/vue-components.html#vue-component-name-in-template-casing) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-component-name-in-template-casing-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-component-name-in-template-casing-good) | テンプレート内のコンポーネント名を指定した形式に揃えます。 |
| [`vue/multi-word-component-names`](https://vizejs.dev/ja/rules/vue-components.html#vue-multi-word-component-names) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-multi-word-component-names-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-multi-word-component-names-good) | コンポーネント名を複数の単語で構成します。 |
| [`vue/no-mutating-props`](https://vizejs.dev/ja/rules/vue-components.html#vue-no-mutating-props) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-mutating-props-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-mutating-props-good) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |
| [`vue/no-non-component-keep-alive-child`](https://vizejs.dev/ja/rules/vue-components.html#vue-no-non-component-keep-alive-child) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-non-component-keep-alive-child-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-non-component-keep-alive-child-good) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |
| [`vue/no-reserved-component-names`](https://vizejs.dev/ja/rules/vue-components.html#vue-no-reserved-component-names) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-reserved-component-names-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-reserved-component-names-good) | 予約済みのコンポーネント名を検出します。 |
| [`vue/no-unused-components`](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-components) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-components-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-components-good) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |
| [`vue/no-unused-properties`](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-properties) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-properties-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-no-unused-properties-good) | defineProps に宣言しているのに使わない prop を検出します。 |
| [`vue/require-component-is`](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-is) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-is-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-is-good) | 動的 component 要素に :is を指定します。 |
| [`vue/require-component-registration`](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-registration) | [悪い](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-registration-bad) · [良い](https://vizejs.dev/ja/rules/vue-components.html#vue-require-component-registration-good) | 使用するコンポーネントを import または登録します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
