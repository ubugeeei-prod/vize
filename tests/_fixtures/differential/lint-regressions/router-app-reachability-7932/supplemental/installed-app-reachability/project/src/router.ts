import { createWebHistory, createRouter } from "vue-router";
import RouteView from "./RouteView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [{ name: "live-user", path: "/users/:id", component: RouteView }],
});
