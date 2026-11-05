---
title: "ecosystem/vue-router-missing-param"
---

# `ecosystem/vue-router-missing-param`

必須のルートパラメーターを省略した名前付き遷移を報告します。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| warning | なし | crossFile と重大度（off/warn/error） |

適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント

アプリの `createApp(...).use(router)` から、ルート定義がそろった登録済みのルーターに到達できる構成が必要です。未確定または動的なルート定義だけでは、名前が未登録だと断定できません。パラメーターを省略しても現在のルートから値を継承する場合があるため、省略についての診断は warning です。

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

## 悪い例

必須の postId がありません。現在のルートに依存する遷移になります。`user-post` の必須パラメーター `postId` を省略しています。Vue Router は現在のルートから値を継承する場合があるため、診断の重大度は `warning` です。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

## 良い例

`userId` と `postId` を両方明示し、現在のルートのパラメーター値に依存せず遷移できるようにします。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[ファイル間ルール一覧](../cross-file.md)
