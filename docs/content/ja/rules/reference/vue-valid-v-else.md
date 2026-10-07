---
title: "vue/valid-v-else"
---

# `vue/valid-v-else`

v-else の位置・引数・値を検査します。

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
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
        "vue/valid-v-else": "error"
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
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

## 良い

```vue
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [全ルール](../all.md)
