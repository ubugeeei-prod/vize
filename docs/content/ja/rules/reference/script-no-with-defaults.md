---
title: "script/no-with-defaults"
---

# `script/no-with-defaults`

Vue 3.5 以降の props 分割代入の既定値を勧めます。

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
        "script/no-with-defaults": "warn"
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
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## 良い

```vue
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [全ルール](../all.md)
