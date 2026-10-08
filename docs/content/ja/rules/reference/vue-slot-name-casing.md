---
title: "vue/slot-name-casing"
---

# `vue/slot-name-casing`

名前付き slot を kebab-case に揃えます。

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
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

名前付きスロット `mySlot` が camelCase で、ハイフン区切りの規約に合いません。

```vue
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

## 良い

`#my-slot` を kebab-case にします。受け取る slot の名前も合わせます。

```vue
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [全ルール](../all.md)
