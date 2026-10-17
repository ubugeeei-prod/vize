---
title: "vue/attribute-order"
---

# `vue/attribute-order`

テンプレートの属性を一定の順に並べます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

構造を決める v-if と通常の id 属性より前に、イベントハンドラーを指定しています。

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

## 良い

v-if、id、イベントハンドラーの順に並べ、ルールの順序に合わせます。

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [全ルール](../all.md)
