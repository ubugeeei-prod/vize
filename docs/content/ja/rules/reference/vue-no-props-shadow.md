---
title: "vue/no-props-shadow"
description: "script setup で prop と同名の宣言を検出します。"
---

# `vue/no-props-shadow`

script setup で prop と同名の宣言を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ローカルの `value` 宣言が同名の prop を隠しています。

```vue annotate="remove:3"
<script setup>
defineProps({ value: String })
const value = "local"
</script>
<template />
```

## 良い例

ローカルのバインディングを別名にします。

```vue annotate="add:3"
<script setup>
defineProps({ value: String })
const localValue = "local"
</script>
<template />
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-props-shadow": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L15)
