---
title: "script/no-deprecated-dollar-scopedslots-api"
---

# `script/no-deprecated-dollar-scopedslots-api`

Vue 3 で $slots に統合された $scopedSlots を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`this.$scopedSlots`、`ctx.$scopedSlots`、裸の `$scopedSlots` 参照が、Vue 3 で削除された Vue 2 の scoped slot API を使っています。

```vue
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## 良い

`$scopedSlots` を `$slots` に置き換え、統合された slot API を使います。この例は削除された API 名の置換を示すもので、参照元の setup コンテキストを作る例ではありません。

```vue
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [全ルール](../all.md)
