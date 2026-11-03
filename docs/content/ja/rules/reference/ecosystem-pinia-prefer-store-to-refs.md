---
title: "ecosystem/pinia-prefer-store-to-refs"
description: "Pinia store の分割代入に storeToRefs() を使います。"
---

# `ecosystem/pinia-prefer-store-to-refs`

Pinia store の分割代入に storeToRefs() を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

store から name を直接分割代入しているため、取り出した値は store の変更に追従しません。

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

## 良い例

store を変数に保持し、storeToRefs で name を store の状態に追従する ref として取り出します。

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
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
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20)
