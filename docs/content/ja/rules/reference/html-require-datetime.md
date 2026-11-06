---
title: "html/require-datetime"
description: "time 要素に機械可読の datetime を指定します。"
---

# `html/require-datetime`

time 要素に機械可読の datetime を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

time 要素には表示用の日付だけがあり、機械可読の datetime 属性がありません。

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

## 良い例

表示した日付に対応する機械可読の日付を、`datetime="2026-05-13"` として指定します。

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/require-datetime": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34)
