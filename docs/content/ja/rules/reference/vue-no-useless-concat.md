---
title: "vue/no-useless-concat"
description: "文字列リテラル同士の不要な連結を検出します。"
---

# `vue/no-useless-concat`

文字列リテラル同士の不要な連結を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

二つの文字列リテラルを実行時に連結しています。

```vue annotate="remove:2"
<template>
  <p>{{ "hello" + " world" }}</p>
</template>
```

## 良い例

同じ文字列を一つのリテラルで表します。

```vue annotate="add:2"
<template>
  <p>{{ "hello world" }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-concat": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L131)
