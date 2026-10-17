---
title: "vue/no-template-lang"
---

# `vue/no-template-lang`

template の lang 指定を避ける方針を示します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の診断を出しません。悪い例・良い例は意図した規約の説明であり、実行時にこのルールの診断が出る例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

<span id="設定できる-id-現在の-sfc-検出なし"></span>

## 設定できる ID（現在の SFC lint では診断されません）

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

良い例は意図する規約を示します。現在の SFC lint では、悪い例・良い例のどちらからもこのルール固有の診断は出ません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [全ルール](../all.md)
