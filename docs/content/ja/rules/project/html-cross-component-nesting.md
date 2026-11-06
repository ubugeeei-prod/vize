---
title: "html/cross-component-nesting"
---

# `html/cross-component-nesting`

import した子コンポーネントの要素を合成して HTML の入れ子を検査します。

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

親の `<p>` の内側に、ルートが `<div>` の子コンポーネントを合成しています。その結果、段落の中にブロック要素が入り、HTML の入れ子が不正になります。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## 良い例

子のブロック要素を含められる `<section>` を親のコンテナーに使います。子コンポーネントは変更しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "html/cross-component-nesting": "warn" },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[ファイル間ルール一覧](../cross-file.md)
