---
title: "a11y/click-events-have-key-events"
description: "クリックで操作する要素にキーボード操作も用意します。"
---

# `a11y/click-events-have-key-events`

クリックで操作する要素にキーボード操作も用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

対話的な役割を持たない通常要素が対象です。button や対話的な ARIA role を持つ要素はこの検出の対象外です。

## 悪い例

通常の `div` にクリックハンドラーだけを指定し、キーボード操作に対応していません。

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

## 良い例

同じ `activate` を button に指定し、標準のキーボード操作を使います。

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
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
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L22)
