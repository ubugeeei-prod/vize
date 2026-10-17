---
title: "vue/html-self-closing"
---

# `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

空のコンポーネントを開始タグと終了タグの組で書き、img と br に設定された自己終了タグの表記を使っていません。

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

## 良い

コンポーネントと void 要素を自己終了タグで書きます。内容がある div は終了タグを残します。

```vue
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [全ルール](../all.md)
