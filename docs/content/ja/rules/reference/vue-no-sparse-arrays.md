---
title: "vue/no-sparse-arrays"
description: "テンプレート式の疎な配列を検出します。"
---

# `vue/no-sparse-arrays`

テンプレート式の疎な配列を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

2 番目の要素がなく、疎な配列になっています。

```vue annotate="remove:2"
<template>
  <p>{{ [1,, 3] }}</p>
</template>
```

## 良い例

すべての配列位置に値を明示します。

```vue annotate="add:2"
<template>
  <p>{{ [1, 2, 3] }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-sparse-arrays": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L124)
