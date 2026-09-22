import { createRouter, createWebHistory } from "vue-router";

export default createRouter({
  history: createWebHistory("/admin/"),
  routes: [
    { path: "/", name: "dashboard" },
    { path: "/members/:memberId", name: "member" },
  ],
});
