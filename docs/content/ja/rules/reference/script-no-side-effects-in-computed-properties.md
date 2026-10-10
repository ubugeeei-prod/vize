---
title: "script/no-side-effects-in-computed-properties"
---

# `script/no-side-effects-in-computed-properties`

Options API の computed getter 内の副作用を検出します。

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
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`doubled` が `this.count` に代入し、`reversed` が `reverse()` で `this.items` を変更しています。どちらも値を導出すべき getter が元の状態を変更しています。

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      this.count = this.count * 2 // side effect: assigns to data
      return this.count
    },
    reversed() {
      return this.items.reverse() // side effect: mutates the array
    }
  }
}
</script>
```

## 良い

`doubled` は代入せず乗算結果を返します。`reversed` は配列をコピーしてから反転し、getter が元のコンポーネント状態を変更しないようにします。

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      return this.count * 2
    },
    reversed() {
      return [...this.items].reverse() // operate on a copy
    }
  }
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [全ルール](../all.md)
