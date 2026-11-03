---
title: "a11y/img-alt"
description: "画像に alt 属性を指定します。装飾画像は空の alt を使います。"
---

# `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

アバター画像に `alt` 属性がなく、代替テキストが指定されていません。

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

## 良い例

`alt="User avatar"` で画像の代わりとなる文字を指定します。

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L17)
