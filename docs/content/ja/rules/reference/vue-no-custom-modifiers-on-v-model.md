---
title: "vue/no-custom-modifiers-on-v-model"
description: "Vue 2 の互換性のため、コンポーネントの v-model に独自の修飾子を使う箇所を検出します。"
---

# `vue/no-custom-modifiers-on-v-model`

Vue 2 の互換性のため、コンポーネントの v-model に独自の修飾子を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

コンポーネントの `v-model` に Vue 2 の互換性制約を適用する、個別に有効化するルールです。修飾子は `lazy`・`number`・`trim` のみ許可します。Vue 3 のコンポーネントは独自の修飾子を実装できるため、Vue 3 のプリセットでは有効になりません。ネイティブ要素はこの検査の対象外です。

## 悪い例

コンポーネントの `.capitalize` は独自の修飾子で、この Vue 2 互換性の制約に違反します。

```vue annotate="remove:1"
<template><MyField v-model.capitalize="value" /></template>
```

## 良い例

独自の修飾子を付けず、通常のコンポーネントの `v-model` を使います。大文字化が必要なら、その処理を明示的に実装します。

```vue annotate="add:1"
<template><MyField v-model="value" /></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-custom-modifiers-on-v-model": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs#L22)
