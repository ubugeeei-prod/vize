---
title: "a11y/no-autofocus"
description: "意図せずフォーカスを移動させる autofocus を検出します。"
---

# `a11y/no-autofocus`

意図せずフォーカスを移動させる autofocus を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

検索の入力欄に autofocus があり、表示時に利用者の操作なしでフォーカスを要求します。

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

## 良い例

autofocus を取り除き、検索欄はそのまま残します。

```vue annotate="add:2"
<template>
  <input name="query" />
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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L21)
