---
title: "script/no-deprecated-destroyed-lifecycle"
description: "Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。"
---

# `script/no-deprecated-destroyed-lifecycle`

Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | あり | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

タイマーの後片付けに、Vue 3 で削除された Vue 2 のライフサイクルオプション `beforeDestroy` を使っています。

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

## 良い例

タイマーの後片付けの処理をそのまま残し、フック名を Vue 3 の `beforeUnmount` に変更します。

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
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
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17)
