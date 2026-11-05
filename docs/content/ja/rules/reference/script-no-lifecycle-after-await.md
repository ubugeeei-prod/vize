---
title: "script/no-lifecycle-after-await"
description: "setup() 内で await の後にライフサイクルフックを登録することを禁止します。"
---

# `script/no-lifecycle-after-await`

setup() 内で await の後にライフサイクルフックを登録することを禁止します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`setup()` で `await loadData()` の後に `onMounted()` を呼んでいます。この時点ではコンポーネントのインスタンスがないため、フックは登録されません。

```vue annotate="remove:5"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    await loadData()
    onMounted(() => {})
  },
}
</script>
```

## 良い例

最初の `await` より前、`setup()` が同期的に実行されている間に `onMounted()` を登録します。

```vue annotate="add:6"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    onMounted(() => {})
    await loadData()
  },
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
        "script/no-lifecycle-after-await": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45)
