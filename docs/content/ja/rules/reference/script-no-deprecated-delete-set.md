---
title: "script/no-deprecated-delete-set"
description: "Vue 3 で削除された $set / $delete を検出します。"
---

# `script/no-deprecated-delete-set`

Vue 3 で削除された $set / $delete を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`created()` が削除された `this.$set` で `ready` を追加しています。

```vue annotate="remove:4"
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## 良い例

同じフィールドを `this.item.ready = true` で直接代入します。

```vue annotate="add:4"
<script lang="ts">
export default {
  created() {
    this.item.ready = true
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
      "script/no-deprecated-delete-set": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L38)
