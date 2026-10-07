import { createWebHistory, createRouter } from "vue-router";
import RouteView from "./RouteView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [{ name: "beta-home", path: "/beta/:id", component: RouteView }],
});
