---
title: "a11y/no-aria-hidden-on-focusable"
---

# `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で隠した指定を検出します。

[悪い例](#悪い) · [良い例](#良い)

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

フォーカスできる Close ボタンを `aria-hidden="true"` でアクセシビリティ ツリーから隠しています。

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

## 良い

ボタンを隠さず、Close の aria-label を指定します。

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全ルール](../all.md)
