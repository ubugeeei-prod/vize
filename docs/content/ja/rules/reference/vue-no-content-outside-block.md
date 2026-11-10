---
title: "vue/no-content-outside-block"
description: "SFC のトップレベルブロック外にある内容を検出します。"
---

# `vue/no-content-outside-block`

SFC のトップレベルブロック外にある内容を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

先頭のテキストが SFC のトップレベルブロック外にあります。

```vue annotate="remove:1,2"
unexpected
<template><div /></template>
```

## 良い例

作成した内容をすべて template ブロック内に置きます。

```vue annotate="add:1"
<template><div>expected</div></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-content-outside-block": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/content.rs#L8)
