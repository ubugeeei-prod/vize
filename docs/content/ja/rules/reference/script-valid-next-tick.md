---
title: "script/valid-next-tick"
---

# `script/valid-next-tick`

nextTick() の完了を await、then、または callback で扱います。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

インポートした `nextTick()` をコールバックなしの式文で呼び、返された Promise を無視しています。DOM 更新後まで待つ処理がありません。

```vue
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## 良い

`await nextTick()` で Promise を使い、後続の setup 処理へ進む前に次の DOM 更新を明示的に待ちます。

```vue
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [全ルール](../all.md)
