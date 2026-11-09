import { createRouter, createWebHistory } from "vue-router";

export const siteRouter = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/:locale/dashboard", name: "dashboard" },
    { path: "/pricing", name: "pricing" },
  ],
});
