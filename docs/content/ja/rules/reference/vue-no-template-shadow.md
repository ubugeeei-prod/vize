---
title: "vue/no-template-shadow"
---

# `vue/no-template-shadow`

テンプレート変数が外側の名前を隠す箇所を検出します。

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の検査は入れ子になった v-for の変数を比較します。script 内の名前と同じという理由だけで単独の v-for を検出するものではありません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
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
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## 良い

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全ルール](../all.md)
