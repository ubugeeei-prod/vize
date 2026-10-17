---
title: "script/no-reserved-keys"
---

# `script/no-reserved-keys`

Options API のキーに Vue の予約名を使う箇所を検出します。

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
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

data の `$el` が Vue 組み込みのインスタンスプロパティと重なり、予約された `$` 接頭辞も使っています。

```vue
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

## 良い

アプリケーションのデータ名を `elementLabel` に変え、組み込みのインスタンス API と予約接頭辞を避けます。

```vue
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30) · [全ルール](../all.md)
