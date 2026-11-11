---
title: "vue/no-restricted-syntax"
description: "設定した構文をテンプレート式で使う箇所を検出します。"
---

# `vue/no-restricted-syntax`

設定した構文をテンプレート式で使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

設定で制限した `CallExpression` がテンプレートにあります。

```vue annotate="remove:2"
<template>
  <p>{{ format(value) }}</p>
</template>
```

## 良い例

テンプレートで関数を呼ばず、準備済みのバインディングを読みます。

```vue annotate="add:2"
<template>
  <p>{{ formattedValue }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-syntax": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-syntax": {
        "selectors": [
          "CallExpression"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/restricted.rs#L20)
