---
title: "vue/enforce-style-attribute"
description: "style ブロックを許可した属性モードに揃えます。"
---

# `vue/enforce-style-attribute`

style ブロックを許可した属性モードに揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

module の style モードは設定した許可一覧にありません。

```vue annotate="remove:2"
<template><div /></template>
<style module>.card { color: red }</style>
```

## 良い例

scoped の style モードはプロジェクト方針で許可されています。

```vue annotate="add:2"
<template><div /></template>
<style scoped>.card { color: red }</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/enforce-style-attribute": "warn"
    },
    "ruleOptions": {
      "vue/enforce-style-attribute": {
        "allow": [
          "scoped"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L35)
