---
title: "vue/no-define-model-default"
description: "defineModel に既定値を指定する箇所を検出します。"
---

# `vue/no-define-model-default`

defineModel に既定値を指定する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

子の model が親と同期しない既定値を宣言しています。

```vue annotate="remove:1,2"
<script setup>
const model = defineModel({ default: "draft" })
</script>
<template />
```

## 良い例

初期値は親から model に渡します。

```vue annotate="add:1,2"
<script setup lang="ts">
const model = defineModel<string>()
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
      "vue/no-define-model-default": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L43)
