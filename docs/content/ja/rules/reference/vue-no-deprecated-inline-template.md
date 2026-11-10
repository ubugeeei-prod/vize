---
title: "vue/no-deprecated-inline-template"
description: "削除済みの inline-template 属性を検出します。"
---

# `vue/no-deprecated-inline-template`

削除済みの inline-template 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

Card に渡す内容に、廃止された inline-template を指定しています。

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

## 良い例

inline-template を取り除き、同じ内容を通常の形で渡します。

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-inline-template": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L21)
