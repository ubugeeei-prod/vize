---
title: "a11y/label-has-for"
---

# `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

離れた label に for がなく、入力欄を囲んでもいないため関連付けがありません。

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

## 良い

`for="email"` を入力欄の ID と一致させて関連付けます。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全ルール](../all.md)
