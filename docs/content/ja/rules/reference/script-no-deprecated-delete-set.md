---
title: "script/no-deprecated-delete-set"
---

# `script/no-deprecated-delete-set`

Vue 3 で削除された $set / $delete を検出します。

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
        "script/no-deprecated-delete-set": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`created()` が削除された `this.$set` で `ready` を追加しています。

```vue
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## 良い

同じフィールドを `this.item.ready = true` で直接代入します。

```vue
<script lang="ts">
export default {
  created() {
    this.item.ready = true
  }
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L41) · [全ルール](../all.md)
