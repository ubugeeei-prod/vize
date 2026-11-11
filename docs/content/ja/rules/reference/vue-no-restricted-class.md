---
title: "vue/no-restricted-class"
description: "設定した class を使う箇所を検出します。"
---

# `vue/no-restricted-class`

設定した class を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

要素が禁止された legacy class を使っています。

```vue annotate="remove:2"
<template>
  <div class="legacy">x</div>
</template>
```

## 良い例

要素は許可された class を使います。

```vue annotate="add:2"
<template>
  <div class="current">x</div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-class": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-class": [
        "legacy"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_class.rs#L13)
