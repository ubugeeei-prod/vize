---
title: "html/require-datetime"
---

# `html/require-datetime`

time 要素に機械可読の datetime を指定します。

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

time 要素には表示用の日付だけがあり、機械可読の datetime 属性がありません。

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

## 良い

表示した日付に対応する機械可読の日付を、`datetime="2026-05-13"` として指定します。

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全ルール](../all.md)
