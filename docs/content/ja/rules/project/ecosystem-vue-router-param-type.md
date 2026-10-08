---
title: "ecosystem/vue-router-param-type"
---

# `ecosystem/vue-router-param-type`

postId は繰り返しパラメーターではないため配列を渡せません。

既定の重大度: error  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

## 悪い

`postId` は一つの値を取る path parameter ですが、配列 `["2"]` を渡しています。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

## 良い

繰り返し指定ではない `postId` の segment に、一つの値 `"2"` を渡します。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](../cross-file.md)
