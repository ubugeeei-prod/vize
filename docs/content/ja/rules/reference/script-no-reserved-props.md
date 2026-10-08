---
title: "script/no-reserved-props"
---

# `script/no-reserved-props`

prop 宣言に Vue の予約名を使う箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

オブジェクト形式の `ref` と `$foo`、配列形式の `key` は予約された prop 名です。`ref` と `key` はフレームワーク用であり、`$` で始まる名前も拒否されます。

```vue
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

## 良い

通常の prop 名 `name` と `refValue` に変え、予約された名前と接頭辞を避けます。

```vue
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [全ルール](../all.md)
