---
title: "script/no-watch-after-await"
---

# `script/no-watch-after-await`

setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。

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
        "script/no-watch-after-await": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`await loadData()` の後に `watch()` を作成し、停止ハンドルも捨てています。このウォッチャーはコンポーネントと一緒に停止されません。

```vue
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

## 良い

最初の `await` より前にウォッチャーを作成するため、コンポーネントのアンマウント時に Vue が停止します。

```vue
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

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44) · [全ルール](../all.md)
