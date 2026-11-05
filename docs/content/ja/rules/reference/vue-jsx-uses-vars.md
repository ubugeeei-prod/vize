---
title: "vue/jsx-uses-vars"
description: "JSX のタグで参照する変数を、コンポーネントの使用として扱います。"
---

# `vue/jsx-uses-vars`

JSX のタグで参照する変数を、コンポーネントの使用として扱います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

悪い例での診断：`vue/no-unused-components`

この補助ルール自体は `vue/jsx-uses-vars` の診断を出しません。`vue/no-unused-components` が使用の有無を判定するとき、JSX のタグに使ったバインディングを使用済みとして扱います。この例では両方のルールを有効にします。`<MyButton.Part />` はルートの `MyButton` を使用済みとします。名前空間付きのタグや `this` のメンバーは、import したコンポーネントの使用として扱いません。

## 悪い例

`MyButton` を import していますが、テンプレートでも JSX でも参照せず、併用する `vue/no-unused-components` が未使用と報告します。

```vue annotate="remove:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <div />;
</script>
<template><div /></template>
```

## 良い例

JSX のタグで import した `MyButton` を参照します。補助ルールがその使用を認識し、併用するルールの未使用コンポーネント診断が出なくなります。

```vue annotate="add:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <MyButton />;
</script>
<template><div /></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/jsx-uses-vars": "error",
        "vue/no-unused-components": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/jsx_uses_vars.rs#L36)
