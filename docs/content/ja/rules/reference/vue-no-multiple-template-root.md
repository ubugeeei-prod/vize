---
title: "vue/no-multiple-template-root"
description: "単一ルートを要求するテンプレートで複数ルートを検出します。"
---

# `vue/no-multiple-template-root`

単一ルートを要求するテンプレートで複数ルートを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

単一ルートを要求する場合に有効にします。通常の Vue 3 は複数ルートを許可します。

## 悪い例

単一ルートを要求する設定で、テンプレートのルートに p が二つ並んでいます。

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

## 良い例

section で囲み、ルートを一つにします。単一ルートを前提とする場合にだけ有効にする規約です。

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multiple-template-root": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L23)
