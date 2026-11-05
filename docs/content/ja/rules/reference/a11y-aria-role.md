---
title: "a11y/aria-role"
description: "抽象ロールを除く、有効な ARIA role を指定します。"
---

# `a11y/aria-role`

抽象ロールを除く、有効な ARIA role を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`datepicker` は有効な ARIA ロールではありません。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

## 良い例

有効な ARIA ロールである `dialog` を使い、日付を選択する領域の名前も指定します。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L23)
