---
title: "script/define-emits-declaration"
---

# `script/define-emits-declaration`

defineEmits を型による宣言形式に揃えます。

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
        "script/define-emits-declaration": "warn"
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
// Runtime array form
const emit = defineEmits(['change', 'update'])

// Runtime object form
const emit = defineEmits({
change: (id: number) => true,
})
</script>
```

## 良い

```vue
<script setup lang="ts">
// Type-based form (preferred)
const emit = defineEmits<{ change: [id: number] }>()

// Named type alias
const emit = defineEmits<Emits>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全ルール](../all.md)
