---
title: "vue/valid-template-root"
description: "Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。"
---

# `vue/valid-template-root`

Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

描画上の役割を与えるディレクティブがない通常の `<template>` を、テンプレートのルートに置いています。

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

## 良い例

描画される `<div>` をルートにします。Vue 3 のフラグメント全般を一つのルートに制限する例ではありません。

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-template-root": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82)
