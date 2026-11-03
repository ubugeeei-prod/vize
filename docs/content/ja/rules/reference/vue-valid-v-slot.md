---
title: "vue/valid-v-slot"
description: "v-slot の適用先・宣言・修飾子を検査します。"
---

# `vue/valid-v-slot`

v-slot の適用先・宣言・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ネイティブの `<div>` にスロットのディレクティブを付けるか、他のデフォルトスロットや名前付きスロットの宣言と競合させています。

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

## 良い例

デフォルトスロットはコンポーネント上、名前付きスロットは子の `<template #header>` に宣言します。

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29)
