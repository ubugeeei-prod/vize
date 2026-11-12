---
title: "vue/prefer-const"
description: "スクリプトとテンプレートが再代入しないバインディングを const にします。"
---

# `vue/prefer-const`

スクリプトとテンプレートが再代入しないバインディングを const にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートは label を読むだけで、スクリプトもテンプレートも再代入していません。let である必要はありません。

```vue annotate="remove:2"
<script setup>
let label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## 良い例

スクリプトとテンプレートが読むだけのバインディングは const にします。

```vue annotate="add:2"
<script setup>
const label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-const": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/prefer_const.rs#L20)
