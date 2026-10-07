---
title: "script/no-deep-destructure-in-props"
---

# `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

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
        "script/no-deep-destructure-in-props": "warn"
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
// Deep nested destructuring
const { user: { name, age } } = defineProps<{ user: User }>()

// Very deep nesting
const { config: { settings: { theme } } } = defineProps()
</script>
```

## 良い

```vue
<script setup lang="ts">
// Simple destructuring (one level)
const { name, count = 0 } = defineProps<{ name: string; count?: number }>()

// Access nested properties in the component instead
const props = defineProps<{ user: User }>()
const userName = computed(() => props.user.name)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [全ルール](../all.md)
