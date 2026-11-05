---
title: "a11y/placeholder-label-option"
description: "required の select を値が空のプレースホルダー option から始めます。"
---

# `a11y/placeholder-label-option`

required の select を値が空のプレースホルダー option から始めます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

select は `required` ですが、最初の option が Japan のため、国が選ばれた状態でフォームが始まり、`required` が送信を止めることはありません。

```vue annotate="remove:4"
<template>
  <select v-model="country" required>
    <option value="jp">Japan</option>
    <option value="fr">France</option>
  </select>
</template>
```

## 良い例

値が空の最初の option がプレースホルダーになり、select は値のない状態から始まるため、`required` によって国の選択が必要になります。

```vue annotate="add:3"
<template>
  <select v-model="country" required>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47)
