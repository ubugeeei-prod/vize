---
title: "vue/v-on-style"
description: "v-on の表記形式を揃えます。"
---

# `vue/v-on-style`

v-on の表記形式を揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`v-on:click` が長い形式で、このルールの省略形に合いません。

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

## 良い例

ハンドラーを変えずに `@click` の省略形を使います。

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
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
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28)
