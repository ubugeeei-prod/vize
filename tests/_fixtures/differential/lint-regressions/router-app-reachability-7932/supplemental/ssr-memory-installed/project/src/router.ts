import { createMemoryHistory, createRouter } from "vue-router";
import RouteView from "./RouteView.vue";

export const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ name: "ssr-user", path: "/users/:id", component: RouteView }],
});
