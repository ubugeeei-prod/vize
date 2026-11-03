---
title: "vue/no-useless-mustaches"
description: "文字列リテラルだけの不要な mustache を検出します。"
---

# `vue/no-useless-mustaches`

文字列リテラルだけの不要な mustache を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

補間の内容が固定の文字列だけで、式の評価が不要です。

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

## 良い例

固定のテキストは直接書きます。変数、値を埋め込むテンプレート文字列、区切りの空白の補間は残します。

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
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
        "vue/no-useless-mustaches": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37)
