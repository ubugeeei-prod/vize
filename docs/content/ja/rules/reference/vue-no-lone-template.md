---
title: "vue/no-lone-template"
---

# `vue/no-lone-template`

不要な template 要素を検出します。

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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

内側の template に、構造や slot を指定する役割がありません。

```vue
<template>
<div><template><p>Details</p></template></div>
</template>
```

## 良い

不要な template を取り除き、div に p を直接入れます。

```vue
<template>
<div><p>Details</p></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [全ルール](../all.md)
