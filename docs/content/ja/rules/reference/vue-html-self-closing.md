---
title: "vue/html-self-closing"
---

# `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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

空のコンポーネントを閉じタグの組で書き、img と br に既定の自己終了の表記を使っていません。

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

## 良い

コンポーネントと void 要素を自己終了にします。内容がある div は閉じタグを残します。

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

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [全ルール](../all.md)
