---
title: "script/prefer-define-emits-binding"
description: "template の $emit ではなく defineEmits から受け取った型付き関数を呼び出します。"
---

# `script/prefer-define-emits-binding`

template の $emit ではなく defineEmits から受け取った型付き関数を呼び出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | あり | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

template がコンポーネント専用の `$emit` ヘルパーを呼び出していますが、`<script setup>` では `defineEmits` が返す型付き関数をすでに受け取っています。

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## 良い例

template から受け取った `emit` を呼び出し、イベント名と payload をコンポーネントの `defineEmits` 契約に通します。

```vue annotate="add:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-emits-binding": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16)
