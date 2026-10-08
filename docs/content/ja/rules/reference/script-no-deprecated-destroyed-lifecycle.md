---
title: "script/no-deprecated-destroyed-lifecycle"
---

# `script/no-deprecated-destroyed-lifecycle`

Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: 対応する検出で利用可能  
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
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

タイマーの後片付けに、Vue 3 で削除された Vue 2 のライフサイクルオプション `beforeDestroy` を使っています。

```vue
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

## 良い

フック名を `beforeUnmount` に変え、後片付けの本体を Vue 3 のライフサイクル名で保ちます。

```vue
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [全ルール](../all.md)
