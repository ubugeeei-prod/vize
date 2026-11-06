---
title: "html/no-consecutive-br"
description: "連続する br 要素による余白指定を検出します。"
---

# `html/no-consecutive-br`

連続する br 要素による余白指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

一つの p の中に br を二つ続けて入れ、ブロック間の余白を作っています。

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

## 良い例

内容を別の p に分け、連続した br を使いません。

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-consecutive-br": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30)
