---
title: "vue/no-deprecated-v-is"
---

# `vue/no-deprecated-v-is`

非推奨の v-is ディレクティブを検出します。

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
        "vue/no-deprecated-v-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Vue 3.1 で非推奨になった `v-is` で、描画するコンポーネントを切り替えています。

```vue
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## 良い

`vue:` プレフィックス付きの `is` 属性、または `<component :is>` で切り替えます。

```vue
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27) · [全ルール](../all.md)
