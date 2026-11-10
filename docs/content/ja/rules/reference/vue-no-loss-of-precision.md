---
title: "vue/no-loss-of-precision"
description: "精度を失う数値リテラルを検出します。"
---

# `vue/no-loss-of-precision`

精度を失う数値リテラルを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

この整数は JavaScript の number として正確に表せません。

```vue annotate="remove:2"
<template>
  <p>{{ 9007199254740993 }}</p>
</template>
```

## 良い例

正確に計算したバインディングを使い、テンプレートの不正確な数値を避けます。

```vue annotate="add:2"
<template>
  <p>{{ preciseValue }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-loss-of-precision": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_loss_of_precision.rs#L12)
