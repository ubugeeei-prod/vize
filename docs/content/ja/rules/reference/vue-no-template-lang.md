---
title: "vue/no-template-lang"
---

# `vue/no-template-lang`

template の lang 指定を検出します。

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "vue/no-template-lang": "warn"
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
<template lang="pug">
div.container
  h1 Hello
</template>
```

## 良い

```vue
<template>
  <div class="container">
    <h1>Hello</h1>
  </div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [全ルール](../all.md)
