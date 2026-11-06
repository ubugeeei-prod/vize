---
title: "vue/no-deprecated-filter"
description: "Vue 2 のパイプによるフィルター構文を検出します。"
---

# `vue/no-deprecated-filter`

Vue 2 のパイプによるフィルター構文を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

capitalize を適用するために、廃止されたフィルターのパイプ構文を使っています。

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

## 良い例

通常の式で capitalize(message) を呼び出します。

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-filter": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56)
