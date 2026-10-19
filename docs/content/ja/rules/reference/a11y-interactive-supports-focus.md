---
title: "a11y/interactive-supports-focus"
---

# `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

span に button のロールとクリックハンドラーを付けても、キーボードでフォーカスできる要素にはなりません。

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## 良い

フォーカスできる標準の button に変更し、同じ `open` を実行します。

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L36) · [全ルール](../all.md)
