---
title: "vue/no-unused-setup-bindings"
description: "script setup に宣言しているのに読み取らない変数を検出します。"
---

# `vue/no-unused-setup-bindings`

script setup に宣言しているのに読み取らない変数を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

script setup の message をテンプレートで使っていません。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

## 良い例

script setup で宣言した message を p の補間で参照し、未使用の宣言を残しません。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-setup-bindings": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19)
