---
title: "vue/valid-v-html"
---

# `vue/valid-v-html`

v-html の値・引数・modifier を検査します。

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
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-html` の式がないか、対応していない引数・修飾子を指定しています。

```vue
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

## 良い

`v-html="html"` で正しい式を渡します。構文が正しくても HTML の無害化や未信頼の内容の安全性は保証されません。

```vue
<template>
<div v-html="html"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [全ルール](../all.md)
