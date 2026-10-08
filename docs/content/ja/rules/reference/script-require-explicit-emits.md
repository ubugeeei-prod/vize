---
title: "script/require-explicit-emits"
---

# `script/require-explicit-emits`

emit するイベントを defineEmits または emits に宣言します。

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
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

受け取った emit 関数が `save` を送信しますが、`defineEmits([])` にはそのイベントが宣言されていません。

```vue
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

## 良い

宣言に `"save"` を追加し、送信する文字列イベントをコンポーネントの明示的なイベント契約に含めます。

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [全ルール](../all.md)
