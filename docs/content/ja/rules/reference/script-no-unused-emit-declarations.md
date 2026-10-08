---
title: "script/no-unused-emit-declarations"
---

# `script/no-unused-emit-declarations`

宣言したまま emit していないイベントを検出します。

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
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`defineEmits` は `change` と `unused` を宣言していますが、受け取った `emit` 関数が送信する文字列イベントは `change` だけです。

```vue
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## 良い

`unused` を除き、イベント宣言を実際の送信にそろえます。この例では emit の参照を外部へ渡していないため、ローカルの使用状況から判断できます。

```vue
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [全ルール](../all.md)
