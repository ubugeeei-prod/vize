---
title: "vue/no-empty-pattern"
description: "テンプレート式の空の分割代入パターンを検出します。"
---

# `vue/no-empty-pattern`

テンプレート式の空の分割代入パターンを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コールバックが空のオブジェクトパターンを分割代入しています。

```vue annotate="remove:2"
<template>
  <p>{{ items.map(({}) => 1) }}</p>
</template>
```

## 良い例

コールバックで使うプロパティを明示します。

```vue annotate="add:2"
<template>
  <p>{{ items.map(({ id }) => id) }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-pattern": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L89)
