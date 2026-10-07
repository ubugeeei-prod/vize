---
title: "vue/multi-word-component-names"
---

# `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

既定の重大度: `error`  
プリセット: `essential`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

検出対象はファイル名です。同じコンポーネントのファイル名を変更します。子要素のタグ名は対象ではありません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`Item.vue`

```vue
<template><p>Item</p></template>
```

## 良い

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [全ルール](../all.md)
