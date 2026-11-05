---
title: "vue/component-definition-name-casing"
description: "コンポーネント定義名を PascalCase または kebab-case に揃えます。"
---

# `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

対象はファイル名です。PascalCase と kebab-case は許可され、混在する形式は検出されます。

## 悪い例

ファイル名 myComponent.vue が先頭の小文字と途中の大文字を混在させ、PascalCase / kebab-case のどちらにもなっていません。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## 良い例

ファイル名を PascalCase の MyComponent.vue に変更します。テンプレートの内容は同じです。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36)
