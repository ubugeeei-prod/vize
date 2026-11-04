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
      "Reports a navigation name that the installed router does not declare.",
      "インストール済みのルーターにない遷移名を報告します。",
    ],
    [
      "extra-param",
      "error",
      '{ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } }',
      "Reports navigation params that the target route does not declare.",
      "対象のルートが宣言していないナビゲーションパラメーターを報告します。",
    ],
    [
      "param-type",
      "error",
      '{ name: "user-post", params: { userId: "1", postId: ["2"] } }',
      "Reports a route param value the declared segment does not accept.",
      "宣言されたパスが受け付けないルートパラメーター値を報告します。",
    ],
    [
      "missing-param",
      "warning",
      '{ name: "user-post", params: { userId: "1" } }',
      "Reports a named navigation that omits a required route param.",
      "必須のルートパラメーターを省略した名前付き遷移を報告します。",
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
