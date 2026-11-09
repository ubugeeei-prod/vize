---
title: "a11y/no-static-element-interactions"
description: "操作部品ではない要素へのイベント指定を検出します。"
---

# `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

操作の役割がない section に Enter キーの操作を指定しています。

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

## 良い例

同じ Enter キーの操作を標準の button に指定し、要素自体に操作の意味を持たせます。

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-static-element-interactions": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L37)
