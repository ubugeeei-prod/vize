---
title: "vue/no-restricted-component-options"
description: "設定したコンポーネントオプションを使う箇所を検出します。"
---

# `vue/no-restricted-component-options`

設定したコンポーネントオプションを使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントが禁止された inheritAttrs を設定しています。

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template>
  <p>ok</p>
</template>
```

## 良い例

コンポーネントは name だけを設定します。

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
      "vue/no-restricted-component-options": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-options": [
        "inheritAttrs"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_options.rs#L12)
