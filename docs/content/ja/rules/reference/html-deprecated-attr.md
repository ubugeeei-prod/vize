---
title: "html/deprecated-attr"
description: "非推奨の HTML 属性を検出します。"
---

# `html/deprecated-attr`

非推奨の HTML 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

文字の配置を指定するために、非推奨の align 属性を p 要素に使っています。

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

## 良い例

クラスと text-align: center で、配置を CSS に移します。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/deprecated-attr": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32)
