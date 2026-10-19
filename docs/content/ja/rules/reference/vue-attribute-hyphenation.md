---
title: "vue/attribute-hyphenation"
---

# `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

コンポーネントの属性名を camelCase の firstName にしています。

```vue
<template>
<UserCard firstName="Ada" />
</template>
```

## 良い

first-name に変更し、設定されたハイフン区切りの属性名の方針に合わせます。

```vue
<template>
<UserCard first-name="Ada" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L32) · [全ルール](../all.md)
