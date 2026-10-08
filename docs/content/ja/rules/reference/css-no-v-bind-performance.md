---
title: "css/no-v-bind-performance"
---

# `css/no-v-bind-performance`

CSS v-bind() の実行時コストを検討するための警告です。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

変化する offset を SFC の CSS の v-bind() で参照しています。

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

## 良い

変化する transform を要素の style バインディングに直接指定します。

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [全ルール](../all.md)
