---
title: "CSS ルール"
description: "`<style>` ブロックのプレーン CSS について、正しさと保守性を検査します。"
---

# CSS ルール

`<style>` ブロックのプレーン CSS について、正しさと保守性を検査します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## より厳しいプリセット (10)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | 表示切り替えに display: none を使う箇所で v-show を検討します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | CSS に直接記述した値を CSS 変数にまとめます。 | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | 詳細度が高い CSS の ID セレクターを検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | 通常のカスケードを上書きする !important を検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | コンポーネント内でユーティリティクラスを定義する箇所を検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | CSS v-bind() の実行時コストを検討するための警告です。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | 書字方向に対応する CSS の論理プロパティを使います。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | 子孫セレクターを CSS のネスト構文でまとめます。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | @font-face に font-display を指定します。 | `opinionated`, `nuxt` |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
