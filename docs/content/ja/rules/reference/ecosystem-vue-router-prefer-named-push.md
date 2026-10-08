---
title: "ecosystem/vue-router-prefer-named-push"
---

# `ecosystem/vue-router-prefer-named-push`

Vue Router のプログラムによる移動に名前付きルートを使います。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

router.push に、現在の URL 表記に結び付く文字列のパスを渡しています。

```vue
<script setup lang="ts">
router.push("/settings");
</script>
```

## 良い

settings のルート名を持つオブジェクトを router.push に渡します。

```vue
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [全ルール](../all.md)
