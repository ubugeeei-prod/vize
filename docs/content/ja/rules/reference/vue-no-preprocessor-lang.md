---
title: "vue/no-preprocessor-lang"
---

# `vue/no-preprocessor-lang`

CSS プリプロセッサーより標準の CSS を使う方針を示します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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

style の lang に SCSS を指定しているため、プリプロセッサーを禁止するルールが記述されたブロックを報告します。

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## 良い

同じ宣言を、プリプロセッサーの lang を指定せずに通常の CSS で記述します。

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11) · [全ルール](../all.md)
