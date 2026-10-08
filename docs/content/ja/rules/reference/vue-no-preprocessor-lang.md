---
title: "vue/no-preprocessor-lang"
---

# `vue/no-preprocessor-lang`

CSS preprocessor より標準の CSS を使う方針を適用します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の検出を生成しません。悪い例・良い例は意図した規約の説明で、実行すると検出される例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

## 設定できる ID（現在の SFC 検出なし）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

style の lang に SCSS を指定しています。preprocessor を使わない規約の例で、現在の SFC 検査はこのルールを生成しません。

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## 良い

同じ CSS から preprocessor の lang を取り除きます。規約の修正例で、現在の実行結果の診断の違いを示すものではありません。

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [全ルール](../all.md)
