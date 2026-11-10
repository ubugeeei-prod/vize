---
title: "vue/valid-v-for"
description: "v-for の式と変数宣言を検査します。"
---

# `vue/valid-v-for`

v-for の式と変数宣言を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

繰り返し式がないか、対応していない `.stop` 修飾子を付けています。

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

## 良い例

`item in items` や `(item, index) of items` の完全な式と、例の key を使います。

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-for": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L39)
