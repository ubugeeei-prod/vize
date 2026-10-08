---
title: "vue/valid-v-on"
---

# `vue/valid-v-on`

v-on のイベント名・式・modifier を検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

イベント引数または必要な handler / object 式がありません。

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

## 良い

イベントと handler を指定するか、引数なしの `v-on` に listener object を渡します。

```vue
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全ルール](../all.md)
