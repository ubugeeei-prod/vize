---
title: "a11y/placeholder-label-option"
---

# `a11y/placeholder-label-option`

required の select を値が空のプレースホルダー option から始めます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

select は `required` ですが、最初の option が Japan のため、国が選ばれた状態でフォームが始まり、`required` が送信を止めることはありません。

```vue
<template>
  <select v-model="country" required>
    <option value="jp">Japan</option>
    <option value="fr">France</option>
  </select>
</template>
```

## 良い

値が空の最初の option がプレースホルダーになり、select は値のない状態から始まるため、`required` によって国の選択が必要になります。

```vue
<template>
  <select v-model="country" required>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47) · [全ルール](../all.md)
