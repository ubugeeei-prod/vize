---
title: "vue/require-macro-variable-name"
description: "コンパイラーマクロの変数名を既定の名前に揃えます。"
---

# `vue/require-macro-variable-name`

コンパイラーマクロの変数名を既定の名前に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`defineProps` の結果を `p` に束縛しています。

```vue annotate="remove:2"
<script setup>
const p = defineProps({ value: String })
</script>
<template />
```

## 良い例

規定の `props` という名前でマクロの結果を示します。

```vue annotate="add:2"
<script setup>
const props = defineProps({ value: String })
</script>
<template />
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-macro-variable-name": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L49)
