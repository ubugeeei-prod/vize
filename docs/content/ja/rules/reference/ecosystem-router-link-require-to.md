---
title: "ecosystem/router-link-require-to"
---

# `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

SFC の単一ルート要素がリンクである場合、親から属性を継承できるため対象外になることがあります。この例は別の要素の内側にあるリンクで、明示的な遷移先が必要です。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

nav の子要素である RouterLink に to がなく、SFC のルート要素を通じた属性の継承も利用できません。

```vue
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## 良い

nav の内側にあるリンクに `to="/settings"` で移動先を明示します。

```vue
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全ルール](../all.md)
