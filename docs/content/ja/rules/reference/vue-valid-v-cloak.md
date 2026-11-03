---
title: "vue/valid-v-cloak"
description: "v-cloak の引数・値・修飾子を検査します。"
---

# `vue/valid-v-cloak`

v-cloak の引数・値・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

値・引数・修飾子を受け取らない `v-cloak` に、それらを指定しています。

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

## 良い例

値のない `v-cloak` を使います。マウント後に Vue が属性を除くまで CSS で非表示にできます。

```vue annotate="add:2"
<template>
<div v-cloak></div>
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
        "vue/valid-v-cloak": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27)
