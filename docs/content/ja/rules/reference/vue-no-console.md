---
title: "vue/no-console"
description: "テンプレート式から console を参照する箇所を検出します。"
---

# `vue/no-console`

テンプレート式から console を参照する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートのイベント式から `console.log` を呼んでいます。

```vue annotate="remove:2"
<template>
  <button @click="console.log(value)">Save</button>
</template>
```

## 良い例

ログ処理を script のバインディングに委ねます。

```vue annotate="add:2"
<template>
  <button @click="logValue">Save</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-console": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_console.rs#L12)
