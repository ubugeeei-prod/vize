---
title: "vue/eqeqeq"
description: "テンプレート式で厳密等価演算子を使います。"
---

# `vue/eqeqeq`

テンプレート式で厳密等価演算子を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

抽象等価はオペランドを暗黙に変換します。

```vue annotate="remove:2"
<template>
  <p>{{ value == null }}</p>
</template>
```

## 良い例

厳密等価は暗黙の変換なしで比較します。

```vue annotate="add:2"
<template>
  <p>{{ value === null }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/eqeqeq": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L90)
