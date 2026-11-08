---
title: "html/no-duplicate-class"
description: "静的 class 属性内のクラス名重複を検出します。"
---

# `html/no-duplicate-class`

静的 class 属性内のクラス名重複を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

静的な class の中に btn が二回あります。

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

## 良い例

btn は一回だけ残し、別の primary と合わせて指定します。

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-duplicate-class": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L34)
