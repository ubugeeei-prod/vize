---
title: "vue/no-unused-script-vars"
description: "スクリプトで宣言しているのに使わない変数、インポート、引数を検出します。"
---

# `vue/no-unused-script-vars`

スクリプトで宣言しているのに使わない変数、インポート、引数を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

script setup の unused を、スクリプトでもテンプレートでも読んでいません。

```vue annotate="remove:4"
<script setup>
const unused = 1;
</script>
<template><p>Welcome</p></template>
```

## 良い例

テンプレートの補間が unused を読むので、宣言したバインディングは使われています。

```vue annotate="add:4"
<script setup>
const unused = 1;
</script>
<template><p>{{ unused }}</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-script-vars": "warn"
    },
    "ruleOptions": {
      "vue/no-unused-script-vars": {
        "varsIgnorePattern": "^_",
        "argsIgnorePattern": "^_"
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/no_unused_script_vars.rs#L24)
