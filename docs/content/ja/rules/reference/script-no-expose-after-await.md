---
title: "script/no-expose-after-await"
description: "await の後の expose() と defineExpose() を禁止します。"
---

# `script/no-expose-after-await`

await の後の expose() と defineExpose() を禁止します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

トップレベルの `await` の後に `defineExpose()` を呼んでいます。この時点ではコンポーネントのインスタンスが確定していないため、何も公開されません。

```vue annotate="remove:2"
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## 良い例

最初の `await` より前、インスタンスが確定している間に `defineExpose()` を呼びます。

```vue annotate="add:3"
<script setup lang="ts">
defineExpose({ reload })
await loadData()
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-expose-after-await": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34)
