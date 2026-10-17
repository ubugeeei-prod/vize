---
title: "vapor/prefer-static-class"
---

# `vapor/prefer-static-class`

文字列リテラルの :class を静的 class に置き換えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

変化しないクラスの文字列をバインディングで評価しています。

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## 良い

同じ panel のクラスを静的な class 属性に指定します。

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [全ルール](../all.md)
