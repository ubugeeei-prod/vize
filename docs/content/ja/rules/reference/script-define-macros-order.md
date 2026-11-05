---
title: "script/define-macros-order"
description: "script setup のコンパイラーマクロを一定の順に宣言します。"
---

# `script/define-macros-order`

script setup のコンパイラーマクロを一定の順に宣言します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`defineEmits` が `defineProps` より先にありますが、`<script setup>` の先頭は `defineProps`、次に `defineEmits` でなければなりません。

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## 良い例

import の直後、他のすべての文より前に `defineProps`、`defineEmits` の順で置きます。その他のマクロの順序は自由です。

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
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
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44)
