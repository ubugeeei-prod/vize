---
title: "vue/no-implicit-coercion"
description: "テンプレート式の暗黙的な型変換を検出します。"
---

# `vue/no-implicit-coercion`

テンプレート式の暗黙的な型変換を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

二重否定で値を暗黙に真偽値へ変換しています。

```vue annotate="remove:2"
<template>
  <p>{{ !!value }}</p>
</template>
```

## 良い例

`Boolean` で変換を明示します。

```vue annotate="add:2"
<template>
  <p>{{ Boolean(value) }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-implicit-coercion": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_implicit_coercion.rs#L12)
