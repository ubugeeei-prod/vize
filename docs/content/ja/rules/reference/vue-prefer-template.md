---
title: "vue/prefer-template"
description: "文字列連結の代わりにテンプレートリテラルを使います。"
---

# `vue/prefer-template`

文字列連結の代わりにテンプレートリテラルを使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

文字列リテラルと `name` を連結しています。

```vue annotate="remove:2"
<template>
  <p>{{ "Hello " + name }}</p>
</template>
```

## 良い例

テンプレートリテラルで補間を一つの文字列にまとめます。

```vue annotate="add:2"
<template>
  <p>{{ `Hello ${name}` }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-template": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L167)
