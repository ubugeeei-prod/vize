---
title: "script/prefer-use-id"
description: "一意な ID の生成に Vue 3.5 の useId() を使います。"
---

# `script/prefer-use-id`

一意な ID の生成に Vue 3.5 の useId() を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`id` に `Math.random()` の値が含まれ、input と label の識別子がサーバーとクライアントの描画で異なる可能性があります。この例では、`id` という変数名から ID の生成箇所だと判定されます。

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## 良い例

Vue 3.5 以降の `useId()` で識別子を作り、`:for` と `:id` は同じ変数を参照し続けます。ランダムな値の生成を取り除きます。

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45)
