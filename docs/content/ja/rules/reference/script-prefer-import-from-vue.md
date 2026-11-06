---
title: "script/prefer-import-from-vue"
description: "内部パッケージではなく vue から import します。"
---

# `script/prefer-import-from-vue`

内部パッケージではなく vue から import します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | あり | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`ref` と `h` を公開パッケージ `vue` ではなく、内部の `@vue/runtime-core` と `@vue/runtime-dom` からインポートしています。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## 良い例

公開ヘルパーは `vue` からインポートします。`vue` が公開しない `isString` は `@vue/shared` を維持します。名前空間・デフォルト・公開と非公開の混在 import は警告のみで、自動修正しません。

```vue annotate="add:2,3"
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-import-from-vue": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36)
