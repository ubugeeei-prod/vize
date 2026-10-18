---
title: "vue/slot-name-casing"
---

# `vue/slot-name-casing`

名前付きスロットを kebab-case に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
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

スロット名を kebab-case の `#my-slot` にします。対応するスロットの出口の名前も合わせます。

```vue
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [全ルール](../all.md)
