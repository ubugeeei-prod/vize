---
title: "vue/object-shorthand"
description: "テンプレート式のオブジェクトに短縮プロパティ記法を使います。"
---

# `vue/object-shorthand`

テンプレート式のオブジェクトに短縮プロパティ記法を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

オブジェクトのキーと値に `value` を重ねて書いています。

```vue annotate="remove:2"
<template>
  <Widget :options="{ value: value }" />
</template>
```

## 良い例

短縮プロパティで同じバインディングを重複なく指定します。

```vue annotate="add:2"
<template>
  <Widget :options="{ value }" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/object-shorthand": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/object_shorthand.rs#L10)
