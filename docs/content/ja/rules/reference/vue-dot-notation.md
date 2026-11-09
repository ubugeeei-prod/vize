---
title: "vue/dot-notation"
description: "テンプレート式で静的なプロパティ参照にドット記法を使います。"
---

# `vue/dot-notation`

テンプレート式で静的なプロパティ参照にドット記法を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

静的な `name` プロパティを角括弧で参照しています。

```vue annotate="remove:2"
<template>
  <p>{{ user["name"] }}</p>
</template>
```

## 良い例

同じ静的プロパティをドット記法で直接参照します。

```vue annotate="add:2"
<template>
  <p>{{ user.name }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/dot-notation": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L61)
