---
title: "a11y/aria-props"
---

# `a11y/aria-props`

存在しない ARIA 属性を検出します。

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`aria-lable` は綴りが誤っており、対応する ARIA 属性ではありません。

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## 良い

正しい `aria-label` に変更してボタンの名前を指定します。

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全ルール](../all.md)
