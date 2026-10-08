---
title: "css/no-display-none"
---

# `css/no-display-none`

表示切り替えに display: none を使う箇所で v-show を検討します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

ローカルの p を `.message` の CSS で非表示にし、テンプレートに表示条件を指定していません。

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

## 良い

同じ p に `v-show="isSaved"` で表示条件を指定し、display: none を取り除きます。

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [全ルール](../all.md)
