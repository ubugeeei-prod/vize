---
title: "html/deprecated-attr"
---

# `html/deprecated-attr`

非推奨の HTML 属性を検出します。

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

文字の配置を指定するために、非推奨の align 属性を p 要素に使っています。

```vue
<template>
<p align="center">Notice</p>
</template>
```

## 良い

クラスと text-align: center で、配置を CSS に移します。

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [全ルール](../all.md)
