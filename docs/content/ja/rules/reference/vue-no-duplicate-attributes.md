---
title: "vue/no-duplicate-attributes"
description: "同じ要素の属性重複を検出します。"
---

# `vue/no-duplicate-attributes`

同じ要素の属性重複を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

一つの button に class を二回指定しています。

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## 良い例

二つのクラスを一つの class 属性にまとめます。

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-duplicate-attributes": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31)
