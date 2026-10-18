---
title: "vue/no-unsafe-url"
---

# `vue/no-unsafe-url`

危険なスキームを含む可能性がある URL 属性やバインディングを検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

a の移動先に、JavaScript を実行する javascript: スキームを使っています。

```vue
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

## 良い

移動先を、通常のローカルパス /next に変更します。

```vue
<template>
<a href="/next">Continue</a>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [全ルール](../all.md)
