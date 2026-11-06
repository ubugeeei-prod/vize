---
title: "a11y/no-i-for-icon"
description: "アイコン用の i 要素を検出し、意味に合う要素を勧めます。"
---

# `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

アイコンの表示に i 要素を使っています。i が持つテキスト上の意味は、アイコンだけのボタンが行う操作を説明するものではありません。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

## 良い例

装飾用の span に aria-hidden を指定してアイコンを支援技術から隠し、別の `Delete item` の文字でボタンの操作を説明します。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-i-for-icon": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35)
