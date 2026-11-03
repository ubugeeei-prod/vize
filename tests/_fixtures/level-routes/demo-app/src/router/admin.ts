import type { RouteRecordRaw } from "vue-router";

export const adminRoutes: RouteRecordRaw[] = [
  {
    path: "/admin",
    name: "admin",
    component: () => import("../views/AdminLayout.vue"),
    children: [{ path: "audit/:entryId", name: "admin-audit", component: () => import("../views/AdminAudit.vue") }],
  },
];
