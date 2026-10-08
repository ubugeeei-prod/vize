---
title: "a11y/anchor-has-content"
---

# `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`/settings` へのリンクが空で、移動先を説明する内容がありません。

```vue
<template>
  <a href="/settings"></a>
</template>
```

## 良い

同じリンクに `Settings` の文字を入れ、移動先を示します。

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全ルール](../all.md)
