---
title: "vue/valid-v-else-if"
description: "v-else-if の位置・引数・値を検査します。"
---

# `vue/valid-v-else-if`

v-else-if の位置・引数・値を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`v-else-if` の直前に条件分岐がない、条件式がない、引数や修飾子を渡している、といった不正な書き方です。

```vue annotate="remove:2,3,4,5"
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## 良い例

先行する `v-if` または `v-else-if` の直後に、条件式を持つ `v-else-if` を置きます。

```vue annotate="add:2,3"
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else-if": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34)
