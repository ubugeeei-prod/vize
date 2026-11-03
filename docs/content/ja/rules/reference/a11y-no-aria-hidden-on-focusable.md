---
title: "a11y/no-aria-hidden-on-focusable"
description: "フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。"
---

# `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

フォーカスできる Close ボタンを `aria-hidden="true"` でアクセシビリティ ツリーから隠しています。

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

## 良い例

aria-hidden を取り除いてボタンを支援技術から参照できる状態にし、aria-label に Close を指定します。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19)
