---
title: "vue/v-bind-style"
---

# `vue/v-bind-style`

v-bind の表記形式を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-bind:class` が長い形式で、設定したコロン省略形の規約に合いません。

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## 良い

同じ式を `:class` にします。値の型ではなくディレクティブの書き方を検査するルールです。

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28) · [全ルール](../all.md)
