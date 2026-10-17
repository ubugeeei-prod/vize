---
title: "vue/no-child-content"
---

# `vue/no-child-content`

v-html / v-text と子コンテンツを同時に指定する箇所を検出します。

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
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

v-text が p の内容を置き換えるため、中に書いたフォールバック用のテキストは表示されません。

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## 良い

子のテキストを取り除き、p の内容を v-text だけで指定します。

```vue
<template>
  <p v-text="message" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [全ルール](../all.md)
