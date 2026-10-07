---
title: "vue/no-multiple-template-root"
---

# `vue/no-multiple-template-root`

単一ルートを要求するテンプレートで複数ルートを検出します。

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

単一ルートを要求する場合に有効にします。通常の Vue 3 は複数ルートを許可します。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-template-root": "error"
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
<p>First</p>
<p>Second</p>
</template>
```

## 良い

```vue
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [全ルール](../all.md)
