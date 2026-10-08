---
title: "vue/component-definition-name-casing"
---

# `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

対象はファイル名です。PascalCase と kebab-case は許可され、混在する形式は検出されます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

ファイル名 myComponent.vue が先頭の小文字と途中の大文字を混在させ、PascalCase / kebab-case のどちらにもなっていません。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## 良い

ファイル名を PascalCase の MyComponent.vue に変更します。テンプレートの内容は同じです。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [全ルール](../all.md)
