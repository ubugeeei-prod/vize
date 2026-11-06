---
title: "vue/no-deprecated-html-element-is"
description: "通常の HTML 要素で旧形式の is を使う箇所を検出します。"
---

# `vue/no-deprecated-html-element-is`

通常の HTML 要素で旧形式の is を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

標準の div で、接頭辞がない旧来の is 属性から Vue コンポーネントを指定しています。

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

## 良い例

動的な component では :is を使い、標準要素では vue: の接頭辞を明示します。

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-html-element-is": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39)
