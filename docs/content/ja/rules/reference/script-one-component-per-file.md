---
title: "script/one-component-per-file"
description: "1 ファイルにコンポーネント定義が複数あることを検出します。"
---

# `script/one-component-per-file`

1 ファイルにコンポーネント定義が複数あることを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

ファイルが `Parent` を export し、`components` の中にもう一つの `Child` を定義しています。

```vue annotate="remove:1,2,3"
<script setup lang="ts">
const First = defineComponent({ name: 'First' })
const Second = defineComponent({ name: 'Second' })
</script>
```

## 良い例

ファイルが定義するのは `Parent` だけです。

```vue annotate="add:1,2,3,4"
<script lang="ts">
export default {
  name: 'Parent'
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
      "script/one-component-per-file": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L26)
