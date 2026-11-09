---
title: "vue/multi-word-component-names"
description: "コンポーネント名を複数の単語で構成します。"
---

# `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

検出対象はファイル名です。同じコンポーネントのファイル名を変更します。子要素のタグ名は対象ではありません。

## 悪い例

Item.vue は一つの語だけのコンポーネント名です。

`Item.vue`

```vue
<template><p>Item</p></template>
```

## 良い例

同じコンポーネントのファイル名を、複数の単語からなる TodoItem.vue に変更します。

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/multi-word-component-names": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
