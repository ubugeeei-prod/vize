---
title: "script/require-default-prop"
description: "省略可能な、Boolean 以外の prop に default を用意します。"
---

# `script/require-default-prop`

省略可能な、Boolean 以外の prop に default を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`name` と `age` は必須でない Boolean 以外の実行時 props で、未指定時の既定値がありません。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

## 良い例

`name` に `default: ''` を付けます。`enabled` は Boolean の暗黙の false を使い、必須の `id` には代替値が不要であるため、二つの除外条件も示しています。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
  }
}
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
        "script/require-default-prop": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58)
