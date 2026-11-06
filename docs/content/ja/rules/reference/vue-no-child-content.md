---
title: "vue/no-child-content"
description: "v-html / v-text と子コンテンツを同時に指定する箇所を検出します。"
---

# `vue/no-child-content`

v-html / v-text と子コンテンツを同時に指定する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

v-text が p の内容を置き換えるため、中に書いたフォールバック用のテキストは表示されません。

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## 良い例

子のテキストを取り除き、p の内容を v-text だけで指定します。

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-child-content": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30)
