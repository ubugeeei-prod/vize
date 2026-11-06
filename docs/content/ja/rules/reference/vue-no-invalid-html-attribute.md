---
title: "vue/no-invalid-html-attribute"
description: "静的 HTML 属性の無効な値を検出します。現在は rel が対象です。"
---

# `vue/no-invalid-html-attribute`

静的 HTML 属性の無効な値を検出します。現在は rel が対象です。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

a の rel に、スタイルシートを参照する link 要素用の stylesheet を指定しています。

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

## 良い例

a の rel に、ヘルプへの参照を表す help を指定します。

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-invalid-html-attribute": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12)
