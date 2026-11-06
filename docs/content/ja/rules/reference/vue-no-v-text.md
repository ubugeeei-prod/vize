---
title: "vue/no-v-text"
description: "v-text の代わりに mustache を使う方針を適用します。"
---

# `vue/no-v-text`

v-text の代わりに mustache を使う方針を適用します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

要素の内容を v-text のディレクティブで指定しています。

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

## 良い例

同じテキストのバインディングを、要素内の mustache に記述します。

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-text": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31)
