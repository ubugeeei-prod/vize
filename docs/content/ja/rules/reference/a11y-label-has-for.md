---
title: "a11y/label-has-for"
description: "label を対象のフォーム部品と関連付けます。"
---

# `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

離れた label に for がなく、入力欄を囲んでもいないため関連付けがありません。

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

## 良い例

`for="email"` を入力欄の ID と一致させて関連付けます。

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L31)
