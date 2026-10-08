---
title: "nuxt/no-page-meta-runtime-values"
---

# `nuxt/no-page-meta-runtime-values`

definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `nuxt`  
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
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`definePageMeta` のオブジェクトを作る際に `useRoute()` を即座に評価しています。メタデータはマクロによって setup の実行時コンテキストの外へ巻き上げられます。

```vue
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

## 良い

`validate` にコールバックを渡し、`useRoute().params.id` の評価をその実行時まで遅らせます。このルールは関数本体内の遅延評価と、メタデータの即時評価を区別します。

```vue
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [全ルール](../all.md)
