---
title: "vapor/require-vapor-attribute"
---

# `vapor/require-vapor-attribute`

Vapor 向けの script setup に vapor 属性を付ける方針を適用します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このルールは callback が空の placeholder です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。

## 設定できる ID（現在の SFC 検出なし）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

script setup に Vapor の指定がありません。これは規約を示す例で、現在の空の callback は診断を生成しません。

```vue
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## 良い

vapor を追加して Vapor のコンパイルを選択します。修正方針を示す例であり、現在の linter がこのルールを検出するという意味ではありません。

```vue
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [全ルール](../all.md)
