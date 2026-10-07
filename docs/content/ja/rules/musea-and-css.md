---
title: Musea と CSS のルール
---

# Musea と CSS のルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| Rule | 目的 |
| --- | --- |
| [`css/no-display-none`](./reference/css-no-display-none.md) | 表示切り替えに display: none を使う箇所で v-show を検討します。 |
| [`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | CSS の直接指定値を CSS 変数にまとめます。 |
| [`css/no-id-selectors`](./reference/css-no-id-selectors.md) | 詳細度が高い CSS の ID セレクターを検出します。 |
| [`css/no-important`](./reference/css-no-important.md) | 通常のカスケードを上書きする !important を検出します。 |
| [`css/no-utility-classes`](./reference/css-no-utility-classes.md) | コンポーネント内で utility class を定義する箇所を検出します。 |
| [`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | CSS v-bind() の実行時コストを検討するための警告です。 |
| [`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | 書字方向に対応する CSS の論理プロパティを使います。 |
| [`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | 子孫セレクターを CSS nesting でまとめます。 |
| [`css/prefer-slotted`](./reference/css-prefer-slotted.md) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 |
| [`css/require-font-display`](./reference/css-require-font-display.md) | @font-face に font-display を指定します。 |
| [`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | 内容のない variant ブロックを検出します。 |
| [`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | 登録した design token に一致する直接指定値を CSS 変数で表現します。 |
| [`musea/require-component`](./reference/musea-require-component.md) | art ブロックに対象の component を指定します。 |
| [`musea/require-title`](./reference/musea-require-title.md) | art ブロックに title を指定します。 |
| [`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | 同じ Art ファイル内の variant 名を一意にします。 |
| [`musea/valid-variant`](./reference/musea-valid-variant.md) | variant ブロックに name を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
