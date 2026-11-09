import { createRouter, createWebHashHistory } from "vue-router";

const routes = [{ path: "/", name: "home" }];

const router = createRouter({ history: createWebHashHistory(), routes });

export default router;
