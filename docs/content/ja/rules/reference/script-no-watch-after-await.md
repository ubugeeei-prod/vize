---
title: "script/no-watch-after-await"
description: "setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。"
---

# `script/no-watch-after-await`

setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`await loadData()` の後に `watch()` を作成し、停止ハンドルも捨てています。このウォッチャーはコンポーネントと一緒に停止されません。

```vue annotate="remove:5"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    await loadData()
    watch(() => props.id, reload)
  },
}
</script>
```

## 良い例

最初の `await` より前にウォッチャーを作成するため、コンポーネントのアンマウント時に Vue が停止します。

```vue annotate="add:6"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    watch(() => props.id, reload)
    await loadData()
  },
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
      "script/no-watch-after-await": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44)
