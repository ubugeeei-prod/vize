---
title: "script/no-deprecated-events-api"
---

# `script/no-deprecated-events-api`

Vue 3 で削除された $on / $off / $once を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
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
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`$on`、`$once`、`$off` の呼び出しが、Vue 3 で削除されたインスタンスのイベントバスメソッドを使っています。

```vue
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

## 良い

有効な `$emit` は維持し、イベントバスへの購読は外部 emitter の `on` に移します。親へのイベント送信と外部イベントバスを別々の API で表します。

```vue
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [全ルール](../all.md)
