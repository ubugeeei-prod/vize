---
title: "a11y/aria-unsupported-elements"
description: "ARIA 属性を使用できない要素への指定を検出します。"
---

# `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ARIA 属性を使えない `meta` に `aria-hidden` を指定しています。

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## 良い例

ARIA 属性だけを取り除き、文字コードの宣言は維持します。

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-unsupported-elements": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L21)
