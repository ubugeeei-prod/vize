import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  createRouteManifest,
  createRouterMatcher,
  defineRoute,
  defineRoutes,
  hydrateRouteState,
  serializeRouteMatch,
} from "./index.ts";

const routes = defineRoutes([
  {
    name: "home",
    path: "/",
    component: "./fixtures/routes/home.vue",
    meta: { title: "Home" },
  },
  defineRoute({
    name: "user",
    path: "/users/:id/:tab?",
    component: "./fixtures/routes/user.vue",
    meta: { title: "User" },
    loader: async ({ params }) => ({ id: params.id }),
  }),
] as const);

void test("matches SSR URLs without browser globals", () => {
  const router = createRouterMatcher(routes);
  const match = router.match("/users/42/settings?from=ssr&tag=a&tag=b#details");

  assert.equal(match?.name, "user");
  assert.deepEqual(match?.params, { id: "42", tab: "settings" });
  assert.deepEqual(match?.query, { from: "ssr", tag: ["a", "b"] });
  assert.equal(match?.hash, "details");
  assert.deepEqual(match?.meta, { title: "User" });
});

void test("resolves typed locations and omits optional params", () => {
  const router = createRouterMatcher(routes);

  assert.equal(router.resolve("home", {}), "/");
  assert.equal(router.resolve("user", { id: "42" }), "/users/42");
  assert.equal(
    router.resolve("user", { id: "42", tab: "activity" }, { query: { page: 2 }, hash: "top" }),
    "/users/42/activity?page=2#top",
  );
});

void test("serializes and hydrates a matched route state", () => {
  const router = createRouterMatcher(routes);
  const match = router.match("https://example.test/users/42?from=server");
  assert.ok(match);

  const serialized = serializeRouteMatch(match, { user: 42 });
  assert.deepEqual(serialized, {
    schemaVersion: 1,
    name: "user",
    pathname: "/users/42",
    params: { id: "42" },
    query: { from: "server" },
    hash: "",
    data: { user: 42 },
  });

  const hydrated = hydrateRouteState(routes, serialized);
  assert.equal(hydrated?.name, "user");
  assert.deepEqual(hydrated?.params, { id: "42" });
  assert.deepEqual(hydrated?.meta, { title: "User" });
});

void test("emits generator metadata from the same route table", () => {
  assert.deepEqual(createRouteManifest(routes), {
    schemaVersion: 1,
    routes: [
      {
        name: "home",
        path: "/",
        component: "./fixtures/routes/home.vue",
        meta: { title: "Home" },
      },
      {
        name: "user",
        path: "/users/:id/:tab?",
        component: "./fixtures/routes/user.vue",
        meta: { title: "User" },
      },
    ],
  });
});

void test("route examples are source-owned Vue files", () => {
  for (const route of routes) {
    const fixture = readFileSync(new URL(`../${route.component}`, import.meta.url), "utf8");
    assert.match(fixture, /<template>/);
    assert.match(fixture, /<script setup lang="ts">/);
  }
});

void test("invalid route tables fail closed with actionable diagnostics", () => {
  assert.throws(
    () =>
      defineRoutes([
        { name: "broken", path: "relative", component: "./fixtures/routes/home.vue" },
      ] as const),
    /VIZE_ROUTER_INVALID_PATH/,
  );
  assert.throws(
    () =>
      defineRoutes([
        { name: "broken", path: "/", component: "./routes/home.ts" as never },
      ] as const),
    /VIZE_ROUTER_COMPONENT_NOT_VUE/,
  );
  assert.throws(
    () =>
      defineRoutes([
        { name: "same", path: "/", component: "./fixtures/routes/home.vue" },
        { name: "same", path: "/other", component: "./fixtures/routes/user.vue" },
      ] as const),
    /VIZE_ROUTER_DUPLICATE_NAME/,
  );
});
