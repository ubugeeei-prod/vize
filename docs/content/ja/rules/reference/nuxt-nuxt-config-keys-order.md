---
title: "nuxt/nuxt-config-keys-order"
---

# `nuxt/nuxt-config-keys-order`

Nuxt 設定のプロパティを推奨順に並べます。

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## 良い

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ modules: [], ssr: true });
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [全ルール](../all.md)
