---
title: Musea と CSS のルール
---

# Musea と CSS のルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`css/no-display-none`](./all.md#css-no-display-none) | [悪い例](./all.md#css-no-display-none-bad) · [良い例](./all.md#css-no-display-none-good) | 表示切り替えに display: none を使う箇所で v-show を検討します。 |
| [`css/no-hardcoded-values`](./all.md#css-no-hardcoded-values) | [悪い例](./all.md#css-no-hardcoded-values-bad) · [良い例](./all.md#css-no-hardcoded-values-good) | CSS の直接指定値を CSS 変数にまとめます。 |
| [`css/no-id-selectors`](./all.md#css-no-id-selectors) | [悪い例](./all.md#css-no-id-selectors-bad) · [良い例](./all.md#css-no-id-selectors-good) | 詳細度が高い CSS の ID セレクターを検出します。 |
| [`css/no-important`](./all.md#css-no-important) | [悪い例](./all.md#css-no-important-bad) · [良い例](./all.md#css-no-important-good) | 通常のカスケードを上書きする !important を検出します。 |
| [`css/no-utility-classes`](./all.md#css-no-utility-classes) | [悪い例](./all.md#css-no-utility-classes-bad) · [良い例](./all.md#css-no-utility-classes-good) | コンポーネント内で utility class を定義する箇所を検出します。 |
| [`css/no-v-bind-performance`](./all.md#css-no-v-bind-performance) | [悪い例](./all.md#css-no-v-bind-performance-bad) · [良い例](./all.md#css-no-v-bind-performance-good) | CSS v-bind() の実行時コストを検討するための警告です。 |
| [`css/prefer-logical-properties`](./all.md#css-prefer-logical-properties) | [悪い例](./all.md#css-prefer-logical-properties-bad) · [良い例](./all.md#css-prefer-logical-properties-good) | 書字方向に対応する CSS の論理プロパティを使います。 |
| [`css/prefer-nested-selectors`](./all.md#css-prefer-nested-selectors) | [悪い例](./all.md#css-prefer-nested-selectors-bad) · [良い例](./all.md#css-prefer-nested-selectors-good) | 子孫セレクターを CSS nesting でまとめます。 |
| [`css/prefer-slotted`](./all.md#css-prefer-slotted) | [悪い例](./all.md#css-prefer-slotted-bad) · [良い例](./all.md#css-prefer-slotted-good) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 |
| [`css/require-font-display`](./all.md#css-require-font-display) | [悪い例](./all.md#css-require-font-display-bad) · [良い例](./all.md#css-require-font-display-good) | @font-face に font-display を指定します。 |
| [`musea/no-empty-variant`](./all.md#musea-no-empty-variant) | [悪い例](./all.md#musea-no-empty-variant-bad) · [良い例](./all.md#musea-no-empty-variant-good) | 内容のない variant ブロックを検出します。 |
| [`musea/prefer-design-tokens`](./all.md#musea-prefer-design-tokens) | [悪い例](./all.md#musea-prefer-design-tokens-bad) · [良い例](./all.md#musea-prefer-design-tokens-good) | 登録した design token に一致する直接指定値を CSS 変数で表現します。 |
| [`musea/require-component`](./all.md#musea-require-component) | [悪い例](./all.md#musea-require-component-bad) · [良い例](./all.md#musea-require-component-good) | art ブロックに対象の component を指定します。 |
| [`musea/require-title`](./all.md#musea-require-title) | [悪い例](./all.md#musea-require-title-bad) · [良い例](./all.md#musea-require-title-good) | art ブロックに title を指定します。 |
| [`musea/unique-variant-names`](./all.md#musea-unique-variant-names) | [悪い例](./all.md#musea-unique-variant-names-bad) · [良い例](./all.md#musea-unique-variant-names-good) | 同じ Art ファイル内の variant 名を一意にします。 |
| [`musea/valid-variant`](./all.md#musea-valid-variant) | [悪い例](./all.md#musea-valid-variant-bad) · [良い例](./all.md#musea-valid-variant-good) | variant ブロックに name を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
