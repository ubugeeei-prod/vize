---
title: "script/no-next-tick"
description: "Vapor 向けコンポーネントの nextTick() 使用を検出します。"
---

# `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。

## 悪い例

Vapor 向けコンポーネントが `nextTick` をインポートして await し、この移行ルールが拒否する DOM 更新待ちの依存を作っています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## 良い例

`useTemplateRef` で input を取得し、`onMounted` でフォーカスします。例の `nextTick` への依存を、明示的なマウント時の処理に置き換えます。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40)
