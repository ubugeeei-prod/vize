---
title: "vue/camelcase"
description: "テンプレート式の識別子を camelCase に揃えます。"
---

# `vue/camelcase`

テンプレート式の識別子を camelCase に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`user_name` には途中のアンダースコアがあります。

```vue annotate="remove:2"
<template>
  <p>{{ user_name }}</p>
</template>
```

## 良い例

`userName` は camelCase です。

```vue annotate="add:2"
<template>
  <p>{{ userName }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/camelcase": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/camelcase.rs#L10)
