---
title: "vue/attribute-hyphenation"
---

# `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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

```vue
<template>
<UserCard firstName="Ada" />
</template>
```

## 良い

```vue
<template>
<UserCard first-name="Ada" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [全ルール](../all.md)
