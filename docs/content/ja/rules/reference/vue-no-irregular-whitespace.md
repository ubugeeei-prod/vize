---
title: "vue/no-irregular-whitespace"
description: "テンプレート式の不規則な空白文字を検出します。"
---

# `vue/no-irregular-whitespace`

テンプレート式の不規則な空白文字を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

式に改行しない空白が含まれています。

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## 良い例

通常のソースコード用空白を使います。

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L103)
