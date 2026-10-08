---
title: "vue/no-deprecated-slot-attribute"
---

# `vue/no-deprecated-slot-attribute`

削除済みの slot 属性を検出します。

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
        "vue/no-deprecated-slot-attribute": "error"
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
<Foo>
<template slot="header"><h1>Title</h1></template>
</Foo>
</template>
```

## 良い

```vue
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L38) · [全ルール](../all.md)
