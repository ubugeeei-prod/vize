---
title: "vue/no-useless-v-bind"
---

# `vue/no-useless-v-bind`

文字列リテラルだけの不要な v-bind を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-v-bind": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

foo に固定の文字列か、補間がない template string をバインドしています。

```vue
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

## 良い

固定の値は静的な属性にし、変数や補間がある値はバインディングを残します。

```vue
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [全ルール](../all.md)
