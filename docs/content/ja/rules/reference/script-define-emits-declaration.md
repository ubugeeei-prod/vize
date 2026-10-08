---
title: "script/define-emits-declaration"
---

# `script/define-emits-declaration`

defineEmits を型による宣言形式に揃えます。

[悪い例](#悪い) · [良い例](#良い)

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

`defineEmits(["change"])` は実行時の配列による宣言です。このスタイルルールは型ベースの宣言を推奨します。

```vue
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

## 良い

`defineEmits<{ change: [id: number] }>()` で宣言を型引数へ移し、`emit("change", 1)` が渡す数値のペイロードも明示します。

```vue
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全ルール](../all.md)
