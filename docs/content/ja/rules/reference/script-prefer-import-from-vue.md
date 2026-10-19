---
title: "script/prefer-import-from-vue"
---

# `script/prefer-import-from-vue`

内部パッケージではなく vue から import します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`ref` と `h` を公開パッケージ `vue` ではなく、内部の `@vue/runtime-core` と `@vue/runtime-dom` からインポートしています。

```vue
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## 良い

公開ヘルパーは `vue` からインポートします。`vue` が公開しない `isString` は `@vue/shared` を維持します。名前空間・デフォルト・公開と非公開の混在 import は警告のみで、自動修正しません。

```vue
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36) · [全ルール](../all.md)
