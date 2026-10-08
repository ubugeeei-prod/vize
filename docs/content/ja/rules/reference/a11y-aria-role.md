---
title: "a11y/aria-role"
---

# `a11y/aria-role`

有効で抽象的ではない ARIA role を指定します。

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`datepicker` は認識される ARIA role ではありません。

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

## 良い

認識される `dialog` を使い、日付を選択する領域の名前も指定します。

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全ルール](../all.md)
