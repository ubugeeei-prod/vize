---
title: "vue/no-script-non-standard-lang"
---

# `vue/no-script-non-standard-lang`

script の非標準の lang 指定を避ける方針を示します。

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
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

lang=coffee で CoffeeScript の構文を使っています。現在の SFC 検査では、この言語に対してこのルール固有の診断は出ません。

```vue
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## 良い

lang=ts と通常の TypeScript の宣言を使い、意図した言語の方針を示します。

```vue
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

良い例は意図する規約を示します。現在の SFC lint では、悪い例・良い例のどちらからもこのルール固有の診断は出ません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [全ルール](../all.md)
