---
title: "script/prefer-use-attrs"
---

# `script/prefer-use-attrs`

setup の context.attrs を useAttrs() に置き換えます。

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
        "script/prefer-use-attrs": "warn"
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
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

## 良い

```vue
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [全ルール](../all.md)
