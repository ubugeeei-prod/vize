---
title: "vue/valid-v-bind"
description: "v-bind の引数・値・修飾子を検査します。"
---

# `vue/valid-v-bind`

v-bind の引数・値・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

引数のない `v-bind` にオブジェクト式がなく、空の引数形式には属性名がありません。

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

## 良い例

属性と式を指定するか、オブジェクトをバインドします。Vue 3.4 以降では `:loading` の同名省略形も使えます。

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-bind": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30)
