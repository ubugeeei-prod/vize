---
title: "vue/no-textarea-mustache"
description: "textarea 内の mustache を検出し、v-model の使用を勧めます。"
---

# `vue/no-textarea-mustache`

textarea 内の mustache を検出し、v-model の使用を勧めます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

textarea の値をバインドせず、要素内のテキスト補間に message を記述しています。

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## 良い例

v-model で、編集可能な textarea の値と message を関連付けます。

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
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
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26)
