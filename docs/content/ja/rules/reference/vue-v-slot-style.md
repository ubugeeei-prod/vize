---
title: "vue/v-slot-style"
---

# `vue/v-slot-style`

v-slot の表記形式を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
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
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

コンポーネントで `#default`、template で `v-slot:header` を使い、場所ごとの規約と逆になっています。

```vue
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

## 良い

コンポーネントの default slot は `v-slot`、template の名前付き slot は `#header` にします。

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全ルール](../all.md)
