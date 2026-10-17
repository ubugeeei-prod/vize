---
title: "a11y/no-role-presentation-on-focusable"
---

# `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

フォーカスできる Billing のリンクに role="presentation" を指定しています。リンク本来の操作可能な役割と矛盾するため、ブラウザーはこの presentation の指定を無視する必要があります。

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## 良い

矛盾する presentation の指定を除き、Billing への標準のリンクの役割を使います。

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全ルール](../all.md)
