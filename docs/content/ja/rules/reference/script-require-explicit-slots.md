---
title: "script/require-explicit-slots"
---

# `script/require-explicit-slots`

useSlots() で使う slot を defineSlots の型で宣言します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
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
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

型付きの `defineProps<{ id: number }>()` から、この例が TypeScript の構文を使っていることが分かります。しかし setup では `defineSlots` を宣言せずに `useSlots()` を使っており、利用するスロットの型が明示されていません。

```vue
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

## 良い

`defineSlots` で `msg: string` を props に持つ `default` slot を宣言し、`useSlots()` と明示的な型付き slot 契約を併記します。

```vue
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [全ルール](../all.md)
