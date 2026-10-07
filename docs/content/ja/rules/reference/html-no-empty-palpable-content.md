---
title: "html/no-empty-palpable-content"
---

# `html/no-empty-palpable-content`

可視コンテンツを期待する要素が空の場合に検出します。

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

## 良い

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [全ルール](../all.md)
