---
title: "vue/max-props"
description: "Vue コンポーネントの props 数が上限を超えた箇所を検出します。"
---

# `vue/max-props`

Vue コンポーネントの props 数が上限を超えた箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントが上限を超える 3 つの props を宣言しています。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ a: number; b: number; c: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## 良い例

コンポーネントは 1 つの prop を宣言します。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ a: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-props": "warn"
    },
    "ruleOptions": {
      "vue/max-props": {
        "maxProps": 2
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_props.rs#L16)
