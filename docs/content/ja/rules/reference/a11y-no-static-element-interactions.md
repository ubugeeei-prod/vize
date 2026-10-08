---
title: "a11y/no-static-element-interactions"
---

# `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

操作の役割がない section に Enter キーの操作を指定しています。

```vue
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

## 良い

同じ操作を標準の button に指定します。

```vue
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全ルール](../all.md)
