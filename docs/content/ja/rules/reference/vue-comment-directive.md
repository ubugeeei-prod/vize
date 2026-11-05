---
title: "vue/comment-directive"
description: "テンプレート内の診断抑制コメントを扱い、設定に応じて未使用のコメントを検出します。"
---

# `vue/comment-directive`

テンプレート内の診断抑制コメントを扱い、設定に応じて未使用のコメントを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

未使用のテンプレート抑制コメントを報告するには、`reportUnusedDisableDirectives: true` が必要です。この例は不要なコメントを削除します。実際に診断を抑制するコメントは使用できます。ルール名を列挙した場合、Vize がその実行で検査するルールだけを使用判定に含めます。

## 悪い例

全ルールを抑制するコメントが、次の `div` で何も抑制していないため、未使用の抑制として報告されます。

```vue annotate="remove:2"
<template>
  <!-- eslint-disable-next-line -->
  <div />
</template>
```

## 良い例

不要なコメントを削除し、要素を変えずに、その行で lint の検査が実行されるようにします。

```vue
<template>
  <div />
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
        "vue/comment-directive": "error"
      },
      "reportUnusedDisableDirectives": true
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/comment_directive.rs#L33)
