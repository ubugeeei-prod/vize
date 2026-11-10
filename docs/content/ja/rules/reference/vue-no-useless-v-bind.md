---
title: "vue/no-useless-v-bind"
description: "文字列リテラルだけの不要な v-bind を検出します。"
---

# `vue/no-useless-v-bind`

文字列リテラルだけの不要な v-bind を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

foo に固定の文字列か、補間がないテンプレート文字列をバインドしています。

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

## 良い例

固定の値は静的な属性にし、変数や補間がある値はバインディングを残します。

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-v-bind": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L34)
