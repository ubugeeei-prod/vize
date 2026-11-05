---
title: "a11y/alt-text"
description: "画像などのメディアに代替テキストを用意します。"
---

# `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

画像の送信ボタンには URL しかなく、操作を説明する `alt` がありません。

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

## 良い例

`alt="Submit search"` を追加し、検索を送信する操作を説明する名前を支援技術に伝えます。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L37)
