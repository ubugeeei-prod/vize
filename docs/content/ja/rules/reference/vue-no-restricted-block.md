---
title: "vue/no-restricted-block"
description: "設定したブロックを使う箇所を検出します。"
---

# `vue/no-restricted-block`

設定したブロックを使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

SFC に禁止された i18n ブロックがあります。

```vue annotate="remove:4"
<template>
  <p>ok</p>
</template>
<i18n lang="json">{}</i18n>
```

## 良い例

SFC は template ブロックだけを残します。

```vue
<template>
  <p>ok</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-block": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-block": [
        "i18n"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/blocks.rs#L14)
