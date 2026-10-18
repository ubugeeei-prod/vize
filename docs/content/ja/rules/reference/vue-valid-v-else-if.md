---
title: "vue/valid-v-else-if"
---

# `vue/valid-v-else-if`

v-else-if の位置・引数・値を検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-else-if` の直前に条件分岐がない、条件式がない、引数や修飾子を渡している、といった不正な書き方です。

```vue
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## 良い

先行する `v-if` または `v-else-if` の直後に、条件式を持つ `v-else-if` を置きます。

```vue
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34) · [全ルール](../all.md)
