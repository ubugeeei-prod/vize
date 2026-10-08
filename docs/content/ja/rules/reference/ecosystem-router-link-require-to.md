---
title: "ecosystem/router-link-require-to"
---

# `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

SFC の単一ルートにあるリンクは、親から属性を継承できるため対象外になる場合があります。この例は明示的な遷移先が必要な内部のリンクです。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

nav の内側の RouterLink に to がなく、ルートの属性継承にも頼れません。

```vue
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## 良い

内部のリンクに `to="/settings"` で移動先を明示します。

```vue
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全ルール](../all.md)
