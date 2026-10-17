---
title: "a11y/no-i-for-icon"
---

# `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

アイコンの表示に i 要素を使っています。i が持つテキスト上の意味は、アイコンだけのボタンが行う操作を説明するものではありません。

```vue
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

## 良い

装飾用の span に aria-hidden を指定してアイコンを支援技術から隠し、別の `Delete item` の文字でボタンの操作を説明します。

```vue
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全ルール](../all.md)
