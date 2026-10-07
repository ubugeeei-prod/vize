---
title: "ecosystem/nuxt-prefer-nuxt-link"
---

# `ecosystem/nuxt-prefer-nuxt-link`

Nuxt の内部リンクに NuxtLink を使います。

既定の重大度: `warning`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
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
<template>
  <a href="/settings">Settings</a>
</template>
```

## 良い

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [全ルール](../all.md)
