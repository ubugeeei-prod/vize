---
title: "Musea と CSS のルール"
---

# Musea と CSS のルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`css/no-display-none`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-display-none) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-display-none-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-display-none-good) | 表示切り替えに display: none を使う箇所で v-show を検討します。 |
| [`css/no-hardcoded-values`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-hardcoded-values) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-hardcoded-values-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-hardcoded-values-good) | CSS の直接指定値を CSS 変数にまとめます。 |
| [`css/no-id-selectors`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-id-selectors) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-id-selectors-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-id-selectors-good) | 詳細度が高い CSS の ID セレクターを検出します。 |
| [`css/no-important`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-important) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-important-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-important-good) | 通常のカスケードを上書きする !important を検出します。 |
| [`css/no-utility-classes`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-utility-classes) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-utility-classes-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-utility-classes-good) | コンポーネント内で utility class を定義する箇所を検出します。 |
| [`css/no-v-bind-performance`](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-v-bind-performance) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-v-bind-performance-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-no-v-bind-performance-good) | CSS v-bind() の実行時コストを検討するための警告です。 |
| [`css/prefer-logical-properties`](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-logical-properties) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-logical-properties-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-logical-properties-good) | 書字方向に対応する CSS の論理プロパティを使います。 |
| [`css/prefer-nested-selectors`](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-nested-selectors) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-nested-selectors-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-nested-selectors-good) | 子孫セレクターを CSS nesting でまとめます。 |
| [`css/prefer-slotted`](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-slotted) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-slotted-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-prefer-slotted-good) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 |
| [`css/require-font-display`](https://vizejs.dev/ja/rules/musea-and-css.html#css-require-font-display) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#css-require-font-display-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#css-require-font-display-good) | @font-face に font-display を指定します。 |
| [`musea/no-empty-variant`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-no-empty-variant) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-no-empty-variant-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-no-empty-variant-good) | 内容のない variant ブロックを検出します。 |
| [`musea/prefer-design-tokens`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-prefer-design-tokens) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-prefer-design-tokens-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-prefer-design-tokens-good) | 登録した design token に一致する直接指定値を CSS 変数で表現します。 |
| [`musea/require-component`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-component) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-component-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-component-good) | art ブロックに対象の component を指定します。 |
| [`musea/require-title`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-title) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-title-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-require-title-good) | art ブロックに title を指定します。 |
| [`musea/unique-variant-names`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-unique-variant-names) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-unique-variant-names-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-unique-variant-names-good) | 同じ Art ファイル内の variant 名を一意にします。 |
| [`musea/valid-variant`](https://vizejs.dev/ja/rules/musea-and-css.html#musea-valid-variant) | [悪い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-valid-variant-bad) · [良い](https://vizejs.dev/ja/rules/musea-and-css.html#musea-valid-variant-good) | variant ブロックに name を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
