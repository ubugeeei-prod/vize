---
title: "vue/no-unsupported-features"
description: "指定した Vue.js の版で未対応の構文を検出します。"
---

# `vue/no-unsupported-features`

指定した Vue.js の版で未対応の構文を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

defineModel は設定した版より新しい Vue を必要とします。

```vue annotate="remove:2,5"
<script setup lang="ts">
const model = defineModel<string>();
</script>
<template>
  <p>{{ model }}</p>
</template>
```

## 良い例

コンポーネントは版で制限されるマクロを使いません。

```vue annotate="add:2,5"
<script setup lang="ts">
const label = "ok";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsupported-features": "warn"
    },
    "ruleOptions": {
      "vue/no-unsupported-features": {
        "version": "^3.3.0"
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/unsupported_features.rs#L19)
