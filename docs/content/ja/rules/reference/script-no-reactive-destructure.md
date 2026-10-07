---
title: "script/no-reactive-destructure"
---

# `script/no-reactive-destructure`

reactive オブジェクトの反応性を失う分割代入を検出します。

既定の重大度: `warning`  
プリセット: _none_  
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
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup lang="ts">
const state = reactive({ count: 0, name: 'foo' })
const { count, name } = state  // loses reactivity!

// Also passing reactive directly to functions that expect refs
someFunction(state.count)  // loses reactivity
</script>
```

## 良い

```vue
<script setup lang="ts">
const state = reactive({ count: 0, name: 'foo' })

// Use toRef or toRefs to maintain reactivity
const count = toRef(state, 'count')
const { count, name } = toRefs(state)

// Or use computed for derived values
const doubleCount = computed(() => state.count * 2)

// Pass refs or computed to functions
someFunction(toRef(state, 'count'))
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [全ルール](../all.md)
