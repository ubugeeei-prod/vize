---
title: "vue/multi-word-component-names"
description: "コンポーネント名を複数の単語で構成します。"
---

# `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

検出対象はファイル名です。完全一致で設定した Item の除外は、別の一語名を許可しません。

## 悪い例

Thing.vue は一語の名前で、設定した除外リストにも含まれていません。

`Thing.vue`

```vue
<template><p>Item</p></template>
```

## 良い例

Item.vue は完全一致で設定した Item の除外により許可されます。

`Item.vue`

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
    },
    "ruleOptions": {
      "vue/multi-word-component-names": {
        "ignores": [
          "Item"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
