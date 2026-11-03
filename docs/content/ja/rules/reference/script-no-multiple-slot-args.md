---
title: "script/no-multiple-slot-args"
description: "scoped slot 関数に複数の引数を渡す箇所を検出します。"
---

# `script/no-multiple-slot-args`

scoped slot 関数に複数の引数を渡す箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

スロットの呼び出しで、複数の位置引数や、個数不明の引数をスプレッド構文で渡しています。Vue のスロットは位置引数の列ではなく、一つの props オブジェクトを受け取ります。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

## 良い例

`{ foo, bar }` でデータを一つの引数にまとめます。`slotProps` を渡す呼び出しと引数を省いた呼び出しも、このルールが許可するスロットの呼び出し方です。

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
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
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61)
