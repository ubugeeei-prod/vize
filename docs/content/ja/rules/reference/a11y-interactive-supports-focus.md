---
title: "a11y/interactive-supports-focus"
---

# `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

span に button の role と click handler を付けても、キーボードでフォーカスできる要素にはなりません。

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## 良い

フォーカスできる標準の button に変更し、同じ `open` を実行します。

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全ルール](../all.md)
