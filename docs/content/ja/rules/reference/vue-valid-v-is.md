---
title: "vue/valid-v-is"
---

# `vue/valid-v-is`

v-is の適用先・引数・値を検査します。

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
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-is` に引数や修飾子を渡す、値がない、ネイティブの HTML 要素ではなくコンポーネントに指定している、といった不正な書き方です。

```vue
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## 良い

`<tr>` などのネイティブ要素に値付きの `v-is` を指定し、代わりに描画するコンポーネントを示します。

```vue
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30) · [全ルール](../all.md)
