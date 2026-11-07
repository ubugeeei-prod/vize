---
title: "vue/no-static-inline-styles"
description: "静的なインライン style 属性を検出します。"
---

# `vue/no-static-inline-styles`

静的なインライン style 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

p の style 属性に、変化しない色を直接指定しています。

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

## 良い例

notice のクラスと scoped の CSS に、変化しない色を移します。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-static-inline-styles": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L19)
