---
title: "vue/prefer-pug-lang"
description: "旧 jade 1.x の template と、pug 3 と意味が異なる記述を検出します。"
---

# `vue/prefer-pug-lang`

旧 jade 1.x の template と、pug 3 と意味が異なる記述を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

template の lang に jade を指定しています。Vue は旧 jade 1.x パッケージで描画し、jade だけの - each も使っています。

```vue annotate="remove:1,3"
<template lang="jade">
ul
  - each item in [1, 2]
    li= item
</template>
```

## 良い例

lang を pug にし、ループを pug 3 の each で記述します。

```vue annotate="add:1,3"
<template lang="pug">
ul
  each item in [1, 2]
    li= item
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-pug-lang": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prefer_pug_lang.rs#L42)
