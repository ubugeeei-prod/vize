import { createRouter, createWebHistory } from "vue-router";
import { generatedRoutes } from "@/generated/routes";

export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/orders/:orderId", name: "order" }, ...generatedRoutes],
});
