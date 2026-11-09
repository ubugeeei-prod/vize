---
title: "script/require-explicit-slots"
description: "useSlots() で使う slot を defineSlots の型で宣言します。"
---

# `script/require-explicit-slots`

useSlots() で使う slot を defineSlots の型で宣言します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

型付きの `defineProps<{ id: number }>()` から、この例が TypeScript の構文を使っていることが分かります。しかし setup では `defineSlots` を宣言せずに `useSlots()` を使っており、利用するスロットの型が明示されていません。

```vue annotate="remove:2,4,5,6"
<script setup lang="ts">
defineSlots<{ default(): unknown }>()
</script>
<template>
  <div><slot /><slot name="footer" /></div>
</template>
```

## 良い例

`defineSlots` で `msg: string` を props に持つ `default` slot を宣言し、`useSlots()` と明示的な型付き slot 契約を併記します。

```vue annotate="add:2,3"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-slots": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L58)
