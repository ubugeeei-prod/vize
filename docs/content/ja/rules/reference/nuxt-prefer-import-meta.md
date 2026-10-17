---
title: "nuxt/prefer-import-meta"
---

# `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応している診断で利用できます  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

`process.client` は旧形式の Nuxt 環境フラグであり、このルールは `import.meta` への移行を求めます。

```vue
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## 良い

`import.meta.client` に置き換え、ブラウザー側だけで実行する分岐を新しい環境フラグで表します。

```vue
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全ルール](../all.md)
