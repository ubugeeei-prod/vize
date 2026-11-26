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
  const match = router.match(new URL("https://example.test/users/42?from=server"));
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

void test("a string location on another origin does not match", () => {
  const router = createRouterMatcher(routes);
  for (const input of [
    "//other.example/users/42",
    "\\\\other.example/users/42",
    "/\t/other.example/users/42",
    "/\\other.example/users/42",
    "https://other.example/users/42",
    "javascript:/users/42",
  ]) {
    assert.equal(router.match(input), undefined, input);
  }
  assert.equal(router.match(new URL("https://app.example/users/42"))?.pathname, "/users/42");
});

void test("a pathname that starts with two slashes does not match", () => {
  const router = createRouterMatcher(
    defineRoutes([
      { name: "any", path: "/*rest", component: "./fixtures/routes/home.vue" },
    ] as const),
  );
  assert.deepEqual(router.match("/a/b")?.params, { rest: "a/b" });
  for (const input of [
    "/..//other.example/a",
    "/%2e%2e//other.example/a",
    new URL("https://app.example//other.example/a"),
  ]) {
    assert.equal(router.match(input), undefined, String(input));
  }
});

void test("a location that cannot be parsed or decoded does not match", () => {
  const router = createRouterMatcher(routes);
  for (const input of ["/users/%E0%A4%A", "//[", "http://["]) {
    assert.equal(router.match(input), undefined, input);
  }
  assert.equal(router.match(new URL("https://app.example/users/%E0%A4%A")), undefined);
});

void test("a resolved path never starts with two slashes", () => {
  const router = createRouterMatcher(
    defineRoutes([
      { name: "any", path: "/*rest", component: "./fixtures/routes/home.vue" },
      { name: "docs", path: "/docs/*rest", component: "./fixtures/routes/home.vue" },
      { name: "pair", path: "/:first/:second?", component: "./fixtures/routes/user.vue" },
    ] as const),
  );

  assert.equal(router.resolve("any", { rest: "/other.example/a" }), "/other.example/a");
  assert.equal(router.resolve("any", { rest: "///other.example//a" }), "/other.example//a");
  assert.equal(router.resolve("docs", { rest: "guide/router" }), "/docs/guide/router");
  assert.throws(
    () => router.resolve("any", { rest: "//" }),
    new Error("[VIZE_ROUTER_MISSING_PARAM] Missing route param rest"),
  );
  assert.throws(
    () => router.resolve("pair", { first: "", second: "other.example" }),
    new Error("[VIZE_ROUTER_MISSING_PARAM] Missing route param first"),
  );
  assert.equal(router.resolve("pair", { first: "a", second: "" }), "/a");
});

void test("a dot segment is not accepted as a param value", () => {
  const router = createRouterMatcher(routes);
  const invalid = (name: string) =>
    new Error(
      `[VIZE_ROUTER_INVALID_PARAM] Route param ${name} cannot contain a "." or ".." segment`,
    );
  for (const id of [".", ".."]) {
    assert.throws(() => router.resolve("user", { id }), invalid("id"), id);
  }
  assert.throws(() => router.resolve("user", { id: "42", tab: ".." }), invalid("tab"));
  assert.equal(router.resolve("user", { id: "..." }), "/users/...");
  assert.equal(router.resolve("user", { id: ".hidden" }), "/users/.hidden");

  const docs = createRouterMatcher(
    defineRoutes([
      { name: "docs", path: "/docs/*rest", component: "./fixtures/routes/home.vue" },
    ] as const),
  );
  for (const rest of ["..", "../users/42", "guide/./router", "guide/.."]) {
    assert.throws(() => docs.resolve("docs", { rest }), invalid("rest"), rest);
  }
  assert.equal(docs.resolve("docs", { rest: "guide/.../router" }), "/docs/guide/.../router");
});
