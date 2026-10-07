---
title: "a11y/aria-unsupported-elements"
---

# `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "a11y/aria-unsupported-elements": "error"
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
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## 良い

```vue
<template>
  <meta charset="utf-8" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全ルール](../all.md)
