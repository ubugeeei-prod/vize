---
title: "script/prefer-use-id"
---

# `script/prefer-use-id`

一意な ID の生成に Vue 3.5 の useId() を使います。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

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

```sh
vp run lint
```

## 悪い

`id` に `Math.random()` の値が含まれ、input と label の識別子がサーバーとクライアントの描画で異なる可能性があります。この例では、`id` という変数名から ID の生成箇所だと判定されます。

```vue
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## 良い

Vue 3.5 以降の `useId()` で識別子を作り、`:for` と `:id` は同じ変数を参照し続けます。ランダムな値の生成を取り除きます。

```vue
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [全ルール](../all.md)
