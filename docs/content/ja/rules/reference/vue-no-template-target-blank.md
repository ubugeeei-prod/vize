---
title: "vue/no-template-target-blank"
---

# `vue/no-template-target-blank`

target=_blank の外部リンクに適切な rel を指定します。

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
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

外部リンクを target=_blank で開きますが、必要な rel の保護がありません。

```vue
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

## 良い

同じリンクに noopener noreferrer を指定します。

```vue
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [全ルール](../all.md)
