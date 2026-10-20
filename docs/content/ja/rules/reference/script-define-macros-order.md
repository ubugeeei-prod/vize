---
title: "script/define-macros-order"
---

# `script/define-macros-order`

script setup のコンパイラーマクロを一定の順に宣言します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`defineEmits` が `defineProps` より先にありますが、`<script setup>` の先頭は `defineProps`、次に `defineEmits` でなければなりません。

```vue
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## 良い

import の直後、他のすべての文より前に `defineProps`、`defineEmits` の順で置きます。その他のマクロの順序は自由です。

```vue
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44) · [全ルール](../all.md)
