---
title: "html/no-dupe-style-properties"
---

# `html/no-dupe-style-properties`

静的 style 属性内のプロパティ重複を検出します。

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "html/no-dupe-style-properties": "warn"
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
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

## 良い

```vue
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [全ルール](../all.md)
