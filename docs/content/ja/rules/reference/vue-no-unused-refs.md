---
title: "vue/no-unused-refs"
description: "テンプレートに宣言しているのに参照しない ref を検出します。"
---

# `vue/no-unused-refs`

テンプレートに宣言しているのに参照しない ref を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートの unused の ref に対応する script の参照がありません。

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

## 良い例

inputEl のテンプレート ref に、script setup の同じ名前の ref を対応させます。

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-refs": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L66)
