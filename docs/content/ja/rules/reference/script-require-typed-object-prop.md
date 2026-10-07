---
title: "script/require-typed-object-prop"
---

# `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

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
        "script/require-typed-object-prop": "warn"
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
const props = defineProps({
foo: Object,
bar: { type: Array },
})
</script>
```

## 良い

```vue
<script setup lang="ts">
const props = defineProps({
foo: Object as PropType<Foo>,
bar: { type: Array as PropType<Bar[]> },
})

// Type-based form carries the element type directly.
const typed = defineProps<{ foo: Foo; bar: Bar[] }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全ルール](../all.md)
