---
title: "script/no-deprecated-dollar-scopedslots-api"
description: "Vue 3 で $slots に統合された $scopedSlots を検出します。"
---

# `script/no-deprecated-dollar-scopedslots-api`

Vue 3 で $slots に統合された $scopedSlots を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`this.$scopedSlots`、`ctx.$scopedSlots`、`this` や `ctx` を付けずに参照した `$scopedSlots` が、Vue 3 で削除された Vue 2 のスコープ付きスロット API を使っています。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## 良い例

`$scopedSlots` を `$slots` に置き換え、統合された slot API を使います。この例は削除された API 名の置換を示すもので、参照元の setup コンテキストを作る例ではありません。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45)
