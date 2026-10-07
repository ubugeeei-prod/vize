import { createMemoryHistory, createRouter } from "vue-router";
export const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ name: "preview-only", path: "/preview", component: { template: "<div />" } }],
});
