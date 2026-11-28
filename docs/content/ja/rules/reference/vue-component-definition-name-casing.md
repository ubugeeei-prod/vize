---
title: "vue/component-definition-name-casing"
description: "コンポーネント定義名を PascalCase または kebab-case に揃えます。"
---

# `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

コンポーネント定義の静的な名前を検査します。name オプション、defineOptions({ name })、app.component() の第 1 引数が対象です。ファイル名は vue/component-file-name-casing が検査します。

## 悪い例

defineOptions で宣言した名前 my_component は単語をアンダースコアで区切っており、PascalCase になっていません。

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "my_component" });
</script>
<template><p>Content</p></template>
```

## 良い例

宣言した名前 MyComponent は既定の PascalCase です。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
<template><p>Content</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/component-definition-name-casing": "PascalCase"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L68)
