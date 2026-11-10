---
title: "vue/slot-name-casing"
description: "描画する slot outlet の静的な名前を設定した形式に揃えます。"
---

# `vue/slot-name-casing`

描画する slot outlet の静的な名前を設定した形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

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
      "vue/slot-name-casing": "camelCase"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L30)
