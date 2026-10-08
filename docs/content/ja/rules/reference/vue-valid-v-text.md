---
title: "vue/valid-v-text"
---

# `vue/valid-v-text`

v-text の値・引数・modifier を検査します。

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
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-text` に文字列の式がないか、対応していない引数・修飾子を指定しています。

```vue
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

## 良い

`v-text="msg"` は構文として有効です。別の `vue/no-v-text` は mustache の使用を推奨する場合があります。

```vue
<template>
<div v-text="msg"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [全ルール](../all.md)
