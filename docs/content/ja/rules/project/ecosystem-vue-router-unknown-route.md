---
title: "ecosystem/vue-router-unknown-route"
---

# `ecosystem/vue-router-unknown-route`

登録済み router に存在しない名前です。

既定の重大度: error  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの `createApp(...).use(router)` から、ルート定義がそろった登録済みのルーターに到達できる構成が必要です。未確定または動的なルート定義だけでは、名前が未登録だと断定できません。パラメーターを省略しても現在のルートから値を継承する場合があるため、省略についての診断は warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

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

到達できる登録済み router の名前は `user-post` ですが、navigation で `user-posts` と誤記しています。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## 良い

二つの path parameter を保ち、登録名の `user-post` に合わせます。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体には、別の診断が適用される場合があります。

[ファイル間ルール一覧](../cross-file.md)
