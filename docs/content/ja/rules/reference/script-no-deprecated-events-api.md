---
title: "script/no-deprecated-events-api"
description: "Vue 3 で削除された $on / $off / $once を検出します。"
---

# `script/no-deprecated-events-api`

Vue 3 で削除された $on / $off / $once を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`$on`、`$once`、`$off` の呼び出しが、Vue 3 で削除されたインスタンスのイベントバスメソッドを使っています。

```vue annotate="remove:4,5,6,7"
<script>
export default {
  mounted() {
    this.$on('event', handler)
    const vm = this
    vm.$once('event', handler)
    this.$off('event', handler)
  }
}
</script>
```

## 良い例

有効な `$emit` は維持し、イベントバスへの購読は外部 emitter の `on` に移します。親へのイベント送信と外部イベントバスを別々の API で表します。

```vue annotate="add:2,3,6,7"
<script>
import mitt from 'mitt'
const emitter = mitt()
export default {
  mounted() {
    this.$emit('event', payload)
    emitter.on('event', handler)
  }
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-events-api": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51)
