---
title: "vue/valid-v-text"
description: "v-text の値・引数・修飾子を検査します。"
---

# `vue/valid-v-text`

v-text の値・引数・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`v-text` に文字列の式がないか、対応していない引数・修飾子を指定しています。

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

## 良い例

`v-text="msg"` は構文として有効です。別の `vue/no-v-text` は mustache の使用を推奨する場合があります。

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
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
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27)
