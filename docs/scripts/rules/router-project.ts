import type { ProjectRuleExample } from "./types.ts";
const common = {
  "index.html": '<div id="app"></div>\n<script type="module" src="/src/main.ts"></script>',
  "src/main.ts":
    'import { createApp } from "vue";\nimport App from "./App.vue";\nimport { router } from "./router";\ncreateApp(App).use(router).mount("#app");',
  "src/App.vue":
    '<script setup lang="ts">\nimport { RouterView } from "vue-router";\n</script>\n<template><RouterView /></template>',
  "src/router.ts":
    'import { createRouter, createWebHistory } from "vue-router";\nimport UserPost from "./UserPost.vue";\nexport const router = createRouter({\n  history: createWebHistory(),\n  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],\n});',
};
export const routerExamples: Record<string, ProjectRuleExample> = Object.fromEntries(
  [
    [
      "unknown-route",
      "error",
      '{ name: "user-posts", params: { userId: "1", postId: "2" } }',
      "The name is absent from the complete installed router.",
      "登録済み router に存在しない名前です。",
    ],
    [
      "extra-param",
      "error",
      '{ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } }',
      "The route does not declare tab; Vue Router discards it.",
      "パスにない tab は Vue Router に破棄されます。",
    ],
    [
      "param-type",
      "error",
      '{ name: "user-post", params: { userId: "1", postId: ["2"] } }',
      "postId is not repeatable, so an array is invalid.",
      "postId は繰り返しパラメーターではないため配列を渡せません。",
    ],
    [
      "missing-param",
      "warning",
      '{ name: "user-post", params: { userId: "1" } }',
      "Required postId is missing; relying on the current route is fragile.",
      "必須の postId がありません。現在のルートに依存する遷移になります。",
    ],
  ].map(([id, severity, bad, note, noteJa]) => {
    const source = (params: string) =>
      `<script setup lang="ts">\nimport { useRouter } from "vue-router";\nconst router = useRouter();\nrouter.push(${params});\n</script>\n<template><p>Post</p></template>`;
    return [
      `ecosystem/vue-router-${id}`,
      {
        shared: common,
        bad: { "src/UserPost.vue": source(bad) },
        good: {
          "src/UserPost.vue": source('{ name: "user-post", params: { userId: "1", postId: "2" } }'),
        },
        severity,
        note,
        noteJa,
      },
    ];
  }),
);
