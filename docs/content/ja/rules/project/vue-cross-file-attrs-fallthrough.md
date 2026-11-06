---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| warning | なし | crossFile と重大度（off/warn/error） |

適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 悪い例

親が `class="notice"` を複数のルートを持つ子コンポーネントに渡しますが、属性の自動適用先がなく、子も `$attrs` を使いません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## 良い例

子が `<main>` に `$attrs` をバインドして属性の適用先を明示します。隣の `<aside>` には適用しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "vue/cross-file-attrs-fallthrough": "warn" },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[ファイル間ルール一覧](../cross-file.md)
