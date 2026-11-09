---
title: "vue/next-tick-style"
description: "nextTick を Promise 形式で使います。"
---

# `vue/next-tick-style`

nextTick を Promise 形式で使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`nextTick` にコールバックを渡しています。

```vue annotate="remove:3"
<script setup>
import { nextTick } from "vue"
nextTick(() => focusInput())
</script>
<template />
```

## 良い例

Promise を await してから処理を続けます。

```vue annotate="add:3,4"
<script setup>
import { nextTick } from "vue"
await nextTick()
focusInput()
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
      "vue/next-tick-style": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L36)
