---
title: "vue/slot-name-casing"
description: "描画する slot outlet の静的な名前を設定したケースに揃えます。"
---

# `vue/slot-name-casing`

描画する slot outlet の静的な名前を設定したケースに揃えます。既定値は `camelCase` です。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | `camelCase`, `kebab-case`, `singleword` |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

描画する slot outlet の静的な name を検査します。親側の v-slot や #name による提供構文は、このルールの対象外です。

## 悪い例

描画する slot outlet の静的な名前 `my-slot` がハイフン区切りで、既定の camelCase 規約に合いません。

```vue annotate="remove:2"
<template>
<MyCard><slot name="my-slot" /></MyCard>
</template>
```

## 良い例

outlet の静的な name を camelCase にします。親側の `#name` による提供構文は、このルールの対象外です。

```vue annotate="add:2"
<template>
<MyCard><slot name="mySlot" /></MyCard>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/slot-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/slot-name-casing": "kebab-case"
    }
  },
});
```

設定したら `vp run lint` を実行します。

`ruleOptions` を省略すると、既定の `camelCase` が使われます。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L28)
