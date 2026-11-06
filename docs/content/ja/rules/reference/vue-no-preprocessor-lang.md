---
title: "vue/no-preprocessor-lang"
description: "CSS プリプロセッサーより標準の CSS を使う方針を示します。"
---

# `vue/no-preprocessor-lang`

CSS プリプロセッサーより標準の CSS を使う方針を示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

style の lang に SCSS を指定しているため、プリプロセッサーを禁止するルールが記述されたブロックを報告します。

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## 良い例

同じ宣言を、プリプロセッサーの lang を指定せずに通常の CSS で記述します。

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-preprocessor-lang": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11)
