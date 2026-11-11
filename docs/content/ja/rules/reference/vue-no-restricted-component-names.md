---
title: "vue/no-restricted-component-names"
description: "設定したコンポーネント名を使う箇所を検出します。"
---

# `vue/no-restricted-component-names`

設定したコンポーネント名を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネント名が禁止された RestrictedShowcase です。

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "RestrictedShowcase" });
</script>
<template>
  <p>ok</p>
</template>
```

## 良い例

コンポーネント名は Widget です。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "Widget" });
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
      "vue/no-restricted-component-names": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-names": [
        "RestrictedShowcase"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_names.rs#L12)
