---
title: "nuxt/no-page-meta-runtime-values"
description: "definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。"
---

# `nuxt/no-page-meta-runtime-values`

definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`definePageMeta` のオブジェクトを作る際に `useRoute()` を即座に評価しています。メタデータはマクロによって setup の実行時コンテキストの外に移されます。

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

## 良い例

`validate` にコールバックを渡し、`useRoute().params.id` の評価をコールバックが呼び出されるまで遅らせます。このルールは関数本体内の遅延評価と、メタデータの即時評価を区別します。

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/no-page-meta-runtime-values": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25)
