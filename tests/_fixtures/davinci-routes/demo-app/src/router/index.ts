import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import { adminRoutes } from "./admin";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    {
      path: "/users/:userId(\\d+)",
      component: () => import("../views/UserLayout.vue"),
      children: [
        { path: "", name: "user", component: () => import("../views/UserProfile.vue") },
        {
          path: "posts/:postId",
          name: "user-post",
          component: () => import("../views/UserPost.vue"),
        },
        { path: "tags/:tags+", name: "user-tags", component: () => import("../views/UserTags.vue") },
      ],
    },
    { path: "/search/:query?", name: "search", component: () => import("../views/SearchView.vue") },
    ...adminRoutes,
  ],
});
