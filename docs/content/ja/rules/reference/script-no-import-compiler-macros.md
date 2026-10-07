---
title: "script/no-import-compiler-macros"
---

# `script/no-import-compiler-macros`

自動的に使える Vue コンパイラーマクロの import を検出します。

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
        "script/no-import-compiler-macros": "error"
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
import { defineProps, defineEmits } from 'vue'
import { withDefaults } from 'vue'

const props = defineProps<Props>()
</script>
```

## 良い

```vue
<script setup lang="ts">
// No import needed - compiler macros are auto-imported
const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Regular imports are fine
import { ref, computed } from 'vue'
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [全ルール](../all.md)
