---
title: "vue/no-template-lang"
---

# `vue/no-template-lang`

template の lang 指定を避ける方針を示します。

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
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

template の lang に Pug を指定しています。HTML のみを使う規約の例で、現在の SFC 検査ではこの ID の診断は出ません。

```vue
<template lang="pug">
p Notice
</template>
```

## 良い

lang を取り除き、通常の HTML の p を直接記述します。規約の例で、現在の SFC 検査で診断が出ることを示すものではありません。

```vue
<template>
<p>Notice</p>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11) · [全ルール](../all.md)
