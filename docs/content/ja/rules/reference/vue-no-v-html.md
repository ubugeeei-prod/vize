---
title: "vue/no-v-html"
description: "XSS を防ぐため、v-html を使用した箇所に警告を出します。"
---

# `vue/no-v-html`

XSS を防ぐため、v-html を使用した箇所に警告を出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

v-html で content を通常の文字ではなく HTML として扱っています。

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

## 良い例

mustache の補間で、HTML を挿入せず content をエスケープしたテキストとして表示します。

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
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
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51)
