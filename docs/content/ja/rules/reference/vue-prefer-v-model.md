---
title: "vue/prefer-v-model"
description: "対応する bind と update リスナーを v-model にまとめます。"
---

# `vue/prefer-v-model`

対応する bind と update リスナーを v-model にまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

bind と update リスナーで同じ値を手動同期しています。

```vue annotate="remove:2"
<template>
  <TextField :model-value="value" @update:model-value="value = $event" />
</template>
```

## 良い例

同じ双方向のコンポーネント規約を `v-model` で一度だけ指定します。

```vue annotate="add:2"
<template>
  <TextField v-model="value" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-v-model": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules.rs#L100)
