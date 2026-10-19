---
title: "script/no-lifecycle-after-await"
---

# `script/no-lifecycle-after-await`

setup() 内で await の後にライフサイクルフックを登録することを禁止します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

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

```sh
vp run lint
```

## 悪い

`setup()` で `await loadData()` の後に `onMounted()` を呼んでいます。この時点ではコンポーネントのインスタンスがないため、フックは登録されません。

```vue
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

## 良い

最初の `await` より前、`setup()` が同期的に実行されている間に `onMounted()` を登録します。

```vue
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45) · [全ルール](../all.md)
