---
title: "script/no-duplicate-attr-inheritance"
description: "fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。"
---

# `script/no-duplicate-attr-inheritance`

fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

自動の属性継承を有効にしたまま、ボタンにも `$attrs` を転送しています。同じ属性が二重に適用される可能性があります。

```vue annotate="remove:1"
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## 良い例

`inheritAttrs: false` で自動の継承を無効にし、ボタンへの明示的な転送だけを残します。

```vue annotate="add:1"
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35)
