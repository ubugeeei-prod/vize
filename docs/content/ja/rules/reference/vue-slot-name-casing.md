---
title: "vue/slot-name-casing"
description: "名前付きスロットを kebab-case に揃えます。"
---

# `vue/slot-name-casing`

名前付きスロットを kebab-case に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

名前付きスロット `mySlot` が camelCase で、ハイフン区切りの規約に合いません。

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

## 良い例

スロット名を kebab-case の `#my-slot` にします。対応するスロットの出口の名前も合わせます。

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
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
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34)
