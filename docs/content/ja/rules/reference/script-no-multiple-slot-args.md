---
title: "script/no-multiple-slot-args"
---

# `script/no-multiple-slot-args`

scoped slot 関数に複数の引数を渡す箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

スロットの呼び出しで、複数の位置引数や、個数不明の引数をスプレッド構文で渡しています。Vue のスロットは位置引数の列ではなく、一つの props オブジェクトを受け取ります。

```vue
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

## 良い

`{ foo, bar }` でデータを一つの引数にまとめます。`slotProps` を渡す呼び出しと引数を省いた呼び出しも、このルールが許可するスロットの呼び出し方です。

```vue
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [全ルール](../all.md)
