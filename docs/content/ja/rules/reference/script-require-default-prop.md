---
title: "script/require-default-prop"
---

# `script/require-default-prop`

任意指定で Boolean ではない prop に default を用意します。

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
        "script/require-default-prop": "error"
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
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

## 良い

```vue
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [全ルール](../all.md)
