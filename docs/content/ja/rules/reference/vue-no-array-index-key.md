---
title: "vue/no-array-index-key"
---

# `vue/no-array-index-key`

v-for の配列インデックスをそのまま key に使う箇所を検出します。

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
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

現在の配列の位置を key に使い、並び替えで項目の識別子が変わります。

```vue
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

## 良い

item.id を key に使い、位置が変わっても項目の識別子を維持します。

```vue
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [全ルール](../all.md)
