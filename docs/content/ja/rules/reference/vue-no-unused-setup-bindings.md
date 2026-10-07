---
title: "vue/no-unused-setup-bindings"
---

# `vue/no-unused-setup-bindings`

script setup に宣言しているのに読み取らない変数を検出します。

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
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

## 良い

```vue
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [全ルール](../all.md)
