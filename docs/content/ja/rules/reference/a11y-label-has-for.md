---
title: "a11y/label-has-for"
description: "label を対象のフォーム部品と関連付けます。"
---

# `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

ネイティブのフォーム部品を描画し、label の入れ子条件を満たすコンポーネント名を指定します。PascalCase と kebab-case は同じ名前として扱います。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。

## 悪い例

離れた label に for がなく、入力欄を囲んでもいないため関連付けがありません。

```vue annotate="remove:2"
<template>
<label>Name</label>
</template>
```

## 良い例

標準の input を label の内側に置き、暗黙的に関連付けます。

```vue annotate="add:2"
<template>
<label>Name <input v-model="name" /></label>
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
    },
    "ruleOptions": {
      "a11y/label-has-for": {
        "controlComponents": [
          "CInputField"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L32)
