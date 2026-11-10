---
title: "vue/no-negated-condition"
description: "否定から始まる三項条件を検出します。"
---

# `vue/no-negated-condition`

否定から始まる三項条件を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

三項演算子が否定条件から始まっています。

```vue annotate="remove:2"
<template>
  <p>{{ !ready ? fallback : primary }}</p>
</template>
```

## 良い例

肯定条件にして分岐の順序を入れ替え、選択を直接表します。

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-condition": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_negated_condition.rs#L12)
