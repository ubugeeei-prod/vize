---
title: "html/id-duplication"
description: "同じテンプレート内の ID 重複を検出します。"
---

# `html/id-duplication`

同じテンプレート内の ID 重複を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

入力欄と説明の p が同じ id="email" を使っているため、label の参照先が一意に定まりません。

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

## 良い例

入力欄は email、説明は email-help に分け、aria-describedby で説明を参照します。

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/id-duplication": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36)
