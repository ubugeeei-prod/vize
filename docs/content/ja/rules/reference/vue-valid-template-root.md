---
title: "vue/valid-template-root"
---

# `vue/valid-template-root`

Vue 3 の fragment に対応する有効なテンプレートルートを検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

描画上の役割を与える directive がない通常の `<template>` を、テンプレートのルートに置いています。

```vue
<template>
  <template>content</template>
</template>
```

## 良い

描画される `<div>` をルートにします。Vue 3 の fragment 全般を一つのルートに制限する例ではありません。

```vue
<template>
  <div>content</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [全ルール](../all.md)
