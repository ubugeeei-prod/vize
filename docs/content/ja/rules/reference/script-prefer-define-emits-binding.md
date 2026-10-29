---
title: "script/prefer-define-emits-binding"
---

# `script/prefer-define-emits-binding`

template の $emit ではなく defineEmits から受け取った型付き関数を呼び出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`

プリセット: _none_

自動修正: 対応している診断で利用できます

適用範囲: Vue SFC の `<script setup>` と、コンパイラーが提供する `defineEmits` の呼び出し。テンプレートの `$emit` 呼び出しを検査します。

オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

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

```sh
vp run lint
```

## 悪い

template がコンポーネント専用の `$emit` ヘルパーを呼び出していますが、`<script setup>` では `defineEmits` が返す型付き関数をすでに受け取っています。

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## 良い

template から受け取った `emit` を呼び出し、イベント名と payload をコンポーネントの `defineEmits` 契約に通します。

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16) · [全ルール](../all.md)
