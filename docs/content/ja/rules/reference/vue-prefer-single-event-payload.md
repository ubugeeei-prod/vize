---
title: "vue/prefer-single-event-payload"
description: "カスタムイベントのペイロードを1つにまとめます。"
---

# `vue/prefer-single-event-payload`

カスタムイベントのペイロードを1つにまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

move イベントが2つのペイロードを宣言して発行しており、インラインのリスナーは x しか受け取れません。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [x: number, y: number] }>()
emit("move", 1, 2)
</script>
<template />
```

## 良い例

move イベントは1つのオブジェクトを渡すので、どのリスナーも両方の座標を受け取ります。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [point: { x: number; y: number }] }>()
emit("move", { x: 1, y: 2 })
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
      "vue/prefer-single-event-payload": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/event_payload.rs#L34)
