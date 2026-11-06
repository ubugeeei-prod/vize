---
title: "vue/html-end-tags"
description: "HTML で省略できる終了タグも明示するよう求めます。"
---

# `vue/html-end-tags`

HTML で省略できる終了タグも明示するよう求めます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

HTML パーサーが省略を許す `li`・`p`・`td` などの終了タグを明示します。必須の終了タグの欠落は `parser/template` が診断します。空要素には終了タグは不要です。

## 悪い例

HTML では `ul` を閉じるときに `li` を暗黙的に閉じられますが、このルールは終了タグの明示を求めます。

```vue annotate="remove:1"
<template><ul><li>First</ul></template>
```

## 良い例

項目の内容の後、親の `ul` を閉じる前に `</li>` を加えます。

```vue annotate="add:1"
<template><ul><li>First</li></ul></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-end-tags": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_end_tags.rs#L30)
