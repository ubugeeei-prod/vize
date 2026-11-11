---
title: "vue/no-restricted-props"
description: "設定した props を使う箇所を検出します。"
---

# `vue/no-restricted-props`

設定した props を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントが禁止された third prop を宣言しています。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ third?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## 良い例

コンポーネントは first を宣言します。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ first?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-props": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-props": [
        "third"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_props.rs#L12)
