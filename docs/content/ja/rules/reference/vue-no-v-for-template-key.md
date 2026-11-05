---
title: "vue/no-v-for-template-key"
description: "Vue 2 の互換性のため、template v-for の key を実体のある子に指定します。"
---

# `vue/no-v-for-template-key`

Vue 2 の互換性のため、template v-for の key を実体のある子に指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

Vue 2 の互換性を検査する、個別に有効化するルールです。Vue 2 では実体のある子に key を置き、Vue 3 では逆に `<template v-for>` に置きます。Vue 3 の規約には `vue/no-v-for-template-key-on-child` を使い、両方を同時に有効にしないでください。

## 悪い例

Vue 2 では `template` 自体が実体のある要素を描画しないため、`<template v-for>` の key は無視されます。

```vue annotate="remove:1"
<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>
```

## 良い例

ここで検査する Vue 2 の規約に従い、繰り返す実体のある子の `li` に項目の key を指定します。

```vue annotate="add:1"
<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key.rs#L33)
