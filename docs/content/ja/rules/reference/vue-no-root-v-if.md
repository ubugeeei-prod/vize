---
title: "vue/no-root-v-if"
description: "テンプレートの単一ルートに v-if を指定する箇所を検出します。"
---

# `vue/no-root-v-if`

テンプレートの単一ルートに v-if を指定する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントのルート自体を v-if で表示・非表示にしています。

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

## 良い例

外側の div をルートとして残し、内側の p に表示条件を指定します。

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
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
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40)
