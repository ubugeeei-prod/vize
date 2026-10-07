---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。

既定の重大度: warning  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。Vue を、Router の例では vue-router もインストールしてください。エントリー ファイルでコンポーネントの関係を明確にしています。

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

## 悪い

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

## 良い

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

良い例はこの検出を避ける修正です。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](../cross-file.md)
