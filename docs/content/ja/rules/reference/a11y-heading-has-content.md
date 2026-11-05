---
title: "a11y/heading-has-content"
description: "見出しに支援技術で読める内容を用意します。"
---

# `a11y/heading-has-content`

見出しに支援技術で読める内容を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`h2` でレベル 2 の見出しを宣言していますが、見出しの内容が空です。

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

## 良い例

同じ `h2` に `Billing settings` の内容を入れます。

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17)
