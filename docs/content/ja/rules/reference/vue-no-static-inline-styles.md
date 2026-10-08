---
title: "vue/no-static-inline-styles"
---

# `vue/no-static-inline-styles`

静的なインライン style 属性を検出します。

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
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

p の style 属性に、変化しない色を直接指定しています。

```vue
<template>
<p style="color: red">Notice</p>
</template>
```

## 良い

notice のクラスと scoped の CSS に、変化しない色を移します。

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [全ルール](../all.md)
