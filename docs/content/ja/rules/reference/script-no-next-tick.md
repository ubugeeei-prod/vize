---
title: "script/no-next-tick"
---

# `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Vapor 向けコンポーネントが `nextTick` をインポートして await し、この移行ルールが拒否する DOM 更新待ちの依存を作っています。

```vue
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## 良い

`useTemplateRef` で input を取得し、`onMounted` でフォーカスします。例の `nextTick` への依存を、明示的なマウント時の処理に置き換えます。

```vue
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全ルール](../all.md)
