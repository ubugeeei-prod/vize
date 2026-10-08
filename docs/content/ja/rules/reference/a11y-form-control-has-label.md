---
title: "a11y/form-control-has-label"
---

# `a11y/form-control-has-label`

フォーム部品に関連付けられたラベルを用意します。

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

検索欄に入力内容を説明する label がありません。

```vue
<template>
  <input type="search" />
</template>
```

## 良い

入力欄を label で囲み、`Search` の文字と関連付けます。

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全ルール](../all.md)
