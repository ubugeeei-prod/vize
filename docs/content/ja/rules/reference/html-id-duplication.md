---
title: "html/id-duplication"
---

# `html/id-duplication`

同じテンプレート内の ID 重複を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

入力欄と説明の p が同じ email の ID を使い、label の参照先が重複しています。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

## 良い

入力欄は email、説明は email-help に分け、aria-describedby で説明を参照します。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [全ルール](../all.md)
