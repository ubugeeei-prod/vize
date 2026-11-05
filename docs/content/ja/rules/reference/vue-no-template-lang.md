---
title: "vue/no-template-lang"
description: "template の lang 指定を避ける方針を示します。"
---

# `vue/no-template-lang`

template の lang 指定を避ける方針を示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

template の lang に Pug を指定しているため、HTML のみを許可するルールが記述されたブロックを報告します。

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

## 良い例

lang を指定せず、通常の HTML の p を直接記述します。

```vue annotate="add:1,2"
<template>
<p>Notice</p>
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
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11)
