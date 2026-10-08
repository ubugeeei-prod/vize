---
title: "nuxt/prefer-import-meta"
---

# `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
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
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## 良い

```vue
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全ルール](../all.md)
