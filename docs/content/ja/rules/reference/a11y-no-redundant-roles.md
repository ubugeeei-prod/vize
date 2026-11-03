---
title: "a11y/no-redundant-roles"
description: "要素本来の意味と重複する ARIA role を検出します。"
---

# `a11y/no-redundant-roles`

要素本来の意味と重複する ARIA role を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

button は元から button の役割を持つため、同じ role を重複して指定しています。

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## 良い例

重複する role を取り除き、HTML の標準の役割を使います。

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
