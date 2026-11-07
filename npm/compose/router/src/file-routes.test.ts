import assert from "node:assert/strict";
import { test } from "node:test";

import { createFileRoutes, routeNameFromFile, routePathFromFile } from "./index.ts";

void test("derives static, dynamic, optional and catch-all file routes", () => {
  const routes = createFileRoutes([
    { file: "./routes/index.vue" },
    { file: "./routes/users/[id].vue", meta: { title: "User" } },
    { file: "./routes/users/[[tab]].vue" },
    { file: "./routes/docs/[...slug].vue" },
  ]);
  assert.deepEqual(
    routes.map(({ name, path, component }) => ({ name, path, component })),
    [
      { name: "index", path: "/", component: "./routes/index.vue" },
      { name: "users-id", path: "/users/:id", component: "./routes/users/[id].vue" },
      {
        name: "users-tab",
        path: "/users/:tab?",
        component: "./routes/users/[[tab]].vue",
      },
      {
        name: "docs-slug",
        path: "/docs/*slug",
        component: "./routes/docs/[...slug].vue",
      },
    ],
  );
});

void test("explicit file-route names and paths override generated values", () => {
  const [route] = createFileRoutes([
    { file: "./routes/account.vue", name: "settings", path: "/settings/profile" },
  ]);
  assert.equal(route?.name, "settings");
  assert.equal(route?.path, "/settings/profile");
});

void test("normalizes route roots and Windows separators deterministically", () => {
  assert.equal(routePathFromFile("src/routes/blog/[id].vue"), "/blog/:id");
  assert.equal(routeNameFromFile("src\\routes\\blog\\[id].vue"), "blog-id");
});
