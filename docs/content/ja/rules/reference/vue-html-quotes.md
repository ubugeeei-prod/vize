---
title: "vue/html-quotes"
---

# `vue/html-quotes`

HTML 属性値の引用符を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

属性の値にシングルクォートを使うか、クォートを省略しています。

```vue
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

## 良い

通常の属性とディレクティブの式をダブルクォートで囲みます。

```vue
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [全ルール](../all.md)
