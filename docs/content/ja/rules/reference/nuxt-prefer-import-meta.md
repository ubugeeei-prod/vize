---
title: "nuxt/prefer-import-meta"
description: "Nuxt の環境フラグを process.* から import.meta.* に置き換えます。"
---

# `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `nuxt` | あり | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`process.client` は旧形式の Nuxt 環境フラグであり、このルールは `import.meta` への移行を求めます。

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## 良い例

`import.meta.client` に置き換え、ブラウザー側だけで実行する分岐を新しい環境フラグで表します。

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/prefer-import-meta": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18)
