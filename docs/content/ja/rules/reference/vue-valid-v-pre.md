---
title: "vue/valid-v-pre"
---

# `vue/valid-v-pre`

v-pre の引数・値・修飾子を検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-v-pre": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

コンパイルを省く印である `v-pre` に、引数・修飾子・値を渡しています。

```vue
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## 良い

値のない `v-pre` でサブツリーをコンパイル対象から外し、マスタッシュをそのまま表示します。

```vue
<template>
<div v-pre>{{ raw }}</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30) · [全ルール](../all.md)
