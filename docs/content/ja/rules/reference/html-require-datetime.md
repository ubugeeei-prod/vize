---
title: "html/require-datetime"
---

# `html/require-datetime`

time 要素に機械可読の datetime を指定します。

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

time に人が読む日付だけがあり、機械が読む datetime がありません。

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

## 良い

`datetime="2026-05-13"` に対応する日付を指定します。

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全ルール](../all.md)
