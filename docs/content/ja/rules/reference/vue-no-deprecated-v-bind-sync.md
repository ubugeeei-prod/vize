---
title: "vue/no-deprecated-v-bind-sync"
description: "削除済みの v-bind の .sync 修飾子を検出します。"
---

# `vue/no-deprecated-v-bind-sync`

削除済みの v-bind の .sync 修飾子を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

.camel と組み合わせたものも含め、廃止された .sync を使っています。

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

## 良い例

一方向なら通常の title のバインディング、更新の受け取りが必要なら v-model:title を使います。

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
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
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42)
