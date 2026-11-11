---
title: "vue/no-restricted-html-elements"
description: "設定した要素を使う箇所を検出します。"
---

# `vue/no-restricted-html-elements`

設定した要素を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートが禁止された marquee 要素を使っています。

```vue annotate="remove:2"
<template>
  <marquee>x</marquee>
</template>
```

## 良い例

テンプレートは p を使います。

```vue annotate="add:2"
<template>
  <p>x</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-html-elements": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-html-elements": [
        "marquee"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_html.rs#L13)
