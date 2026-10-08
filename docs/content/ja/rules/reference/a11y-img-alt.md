---
title: "a11y/img-alt"
---

# `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

アバター画像に `alt` 属性がなく、画像の代替テキストを確認できません。

```vue
<template>
  <img src="/avatar.png" />
</template>
```

## 良い

`alt="User avatar"` で画像の代わりとなる文字を指定します。

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全ルール](../all.md)
