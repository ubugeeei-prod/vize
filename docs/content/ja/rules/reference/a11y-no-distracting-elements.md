---
title: "a11y/no-distracting-elements"
description: "marquee や blink などの注意をそらす要素を検出します。"
---

# `a11y/no-distracting-elements`

marquee や blink などの注意をそらす要素を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

marquee を使って文字を自動的に動かしています。

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

## 良い例

同じ案内を p に入れ、自動的に動く要素を使いません。

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-distracting-elements": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16)
