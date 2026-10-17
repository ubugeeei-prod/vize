---
title: "vue/no-preprocessor-lang"
---

# `vue/no-preprocessor-lang`

CSS プリプロセッサーより標準の CSS を使う方針を示します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の診断を出しません。悪い例・良い例は意図した規約の説明であり、実行時にこのルールの診断が出る例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

<span id="設定できる-id-現在の-sfc-検出なし"></span>

## 設定できる ID（現在の SFC lint では診断されません）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

style の lang に SCSS を指定しています。プリプロセッサーを使わない規約の例で、現在の SFC 検査ではこのルール固有の診断は出ません。

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## 良い

同じ CSS からプリプロセッサーの lang を取り除きます。規約の修正例で、現在の SFC 検査で診断の有無に差が出る例ではありません。

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は意図する規約を示します。現在の SFC lint では、悪い例・良い例のどちらからもこのルール固有の診断は出ません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [全ルール](../all.md)
