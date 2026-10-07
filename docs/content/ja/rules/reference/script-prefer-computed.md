---
title: "script/prefer-computed"
---

# `script/prefer-computed`

他の状態から導ける値を watcher で同期する代わりに computed で表現します。

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

派生値だけを代入する watcher が対象です。ユーザーが編集するコピーや別の副作用を持つ処理は対象外です。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
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
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

## 良い

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [全ルール](../all.md)
