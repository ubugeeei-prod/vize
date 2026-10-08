---
title: "type/no-unsafe-template-binding"
---

# `type/no-unsafe-template-binding`

テンプレートで安全でない型の値を使用する箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

## 悪い

補間する `value` の型を明示的に `any` とし、チェッカーがテンプレートの binding を安全な具体的な型として確認できません。

```vue
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

## 良い

型注釈を `string` に変え、描画する値を変えずに、同じ補間へ検査できる具体的な型を与えます。

```vue
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [全ルール](../all.md)
