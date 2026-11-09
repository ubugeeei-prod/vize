---
title: "vue/sort-keys"
description: "コンポーネントオプション内のキーを設定した順序に並べます。"
---

# `vue/sort-keys`

コンポーネントオプション内のキーを設定した順序に並べます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ネストしたメソッドのキーが昇順ではありません。

```vue annotate="remove:2"
<script>
export default { methods: { zebra() {}, alpha() {} } }
</script>
<template />
```

## 良い例

ネストしたメソッドのキーを設定した順序に並べます。

```vue annotate="add:2"
<script>
export default { methods: { alpha() {}, zebra() {} } }
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
      "vue/sort-keys": "warn"
    },
    "ruleOptions": {
      "vue/sort-keys": {
        "order": "asc",
        "caseSensitive": true,
        "natural": false
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L23)
