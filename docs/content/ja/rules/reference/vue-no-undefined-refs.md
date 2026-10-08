---
title: "vue/no-undefined-refs"
---

# `vue/no-undefined-refs`

テンプレート内の未定義変数参照を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

script は message しか宣言していませんが、テンプレートが missing を参照しています。

```vue
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## 良い

宣言済みの message を補間で参照します。

```vue
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [全ルール](../all.md)
