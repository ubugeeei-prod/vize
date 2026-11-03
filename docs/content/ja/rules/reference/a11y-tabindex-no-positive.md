---
title: "a11y/tabindex-no-positive"
description: "通常のフォーカス順序を変える正の tabindex を検出します。"
---

# `a11y/tabindex-no-positive`

通常のフォーカス順序を変える正の tabindex を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

tabindex に正の値 3 を指定し、通常の操作要素より先にフォーカスが移動するようにしています。

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

## 良い例

正の tabindex を取り除き、button の標準のフォーカス順を使います。

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16)
