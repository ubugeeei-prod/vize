---
title: "a11y/heading-levels"
description: "見出しの階層を飛ばした指定を検出します。"
---

# `a11y/heading-levels`

見出しの階層を飛ばした指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

見出しが `h1` から `h3` に飛び、レベル 2 を省略しています。

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

## 良い例

請求設定の見出しを `h2` にし、階層を順に並べます。

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34)
