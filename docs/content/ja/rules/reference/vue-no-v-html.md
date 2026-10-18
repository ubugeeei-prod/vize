---
title: "vue/no-v-html"
---

# `vue/no-v-html`

XSS を防ぐため、v-html を使用した箇所に警告を出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
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
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

v-html で content を通常の文字ではなく HTML として扱っています。

```vue
<template>
  <article v-html="content" />
</template>
```

## 良い

mustache の補間で、HTML を挿入せず content をエスケープしたテキストとして表示します。

```vue
<template>
  <article>{{ content }}</article>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [全ルール](../all.md)
