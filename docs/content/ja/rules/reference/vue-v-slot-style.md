---
title: "vue/v-slot-style"
---

# `vue/v-slot-style`

v-slot の表記形式を揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

コンポーネントのデフォルトスロットは `v-slot`、template の名前付きスロットは `#header` にします。

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全ルール](../all.md)
