---
title: "vue/no-unsandboxed-iframe"
---

# `vue/no-unsandboxed-iframe`

iframe に sandbox 属性を指定します。

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
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

iframe に sandbox がなく、機能の制限を指定していません。

```vue
<template>
<iframe src="/embed"></iframe>
</template>
```

## 良い

sandbox で制限し、script が必要な場合にだけ allow-scripts を明示します。

```vue
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [全ルール](../all.md)
