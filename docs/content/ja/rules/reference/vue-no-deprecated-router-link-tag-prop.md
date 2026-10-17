---
title: "vue/no-deprecated-router-link-tag-prop"
---

# `vue/no-deprecated-router-link-tag-prop`

router-link の削除済み tag prop を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

RouterLink の廃止された tag prop で button を指定しています。

```vue
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

## 良い

スロットから navigate を受け取り、明示的に記述した button のクリック時に呼び出します。

```vue
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [全ルール](../all.md)
