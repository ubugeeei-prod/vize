---
title: "a11y/alt-text"
---

# `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

画像の送信ボタンには URL しかなく、操作を説明する `alt` がありません。

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

## 良い

`alt="Submit search"` を追加し、検索を送信する操作を説明する名前を支援技術に伝えます。

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全ルール](../all.md)
