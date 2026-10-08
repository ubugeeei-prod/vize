---
title: "vue/valid-attribute-name"
---

# `vue/valid-attribute-name`

有効な属性名を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

悪い例での診断: `parser/template`

不正な属性名は、この防御的なルールに届く前に parser/template で検出されます。悪い例で確認するのは parser/template の検出で、vue/valid-attribute-name が別に出るとは限りません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`my"attr` の引用符で属性名が壊れています。この例の診断は `parser/template` で、別のルール診断の生成を約束するものではありません。

```vue
<template>
<div my"attr="value"></div>
</template>
```

## 良い

`my-attr` は正しい属性名で、テンプレート parser が属性と値を読み取れます。

```vue
<template>
<div my-attr="value"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [全ルール](../all.md)
