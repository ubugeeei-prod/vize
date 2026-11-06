---
title: "vue/v-bind-style"
description: "v-bind の表記形式を揃えます。"
---

# `vue/v-bind-style`

v-bind の表記形式を揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`v-bind:class` が長い形式で、設定で指定したコロンによる省略形の規約に合いません。

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## 良い例

同じ式を `:class` にします。値の型ではなくディレクティブの書き方を検査するルールです。

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-bind-style": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28)
