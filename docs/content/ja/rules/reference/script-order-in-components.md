---
title: "script/order-in-components"
description: "コンポーネントオプションのプロパティ順を既定の並びに揃えます。"
---

# `script/order-in-components`

コンポーネントオプションのプロパティ順を既定の並びに揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`data` が `props` より前に書かれており、後ろのグループが前のグループより上にあります。

```vue annotate="remove:5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  props: ['label']
}
</script>
```

## 良い例

`props` が `data` より前にあり、既定のグループ順と一致します。

```vue annotate="add:3,6"
<script lang="ts">
export default {
  props: ['label'],
  data() {
    return { count: 0 }
  }
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/order-in-components": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/order_in_components.rs#L35)
