import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createRouterMatcher,
  defineRoutes,
  hydrateRouteState,
  serializeRouteMatch,
} from "./index.ts";

const routes = defineRoutes([
  { name: "home", path: "/", component: "./routes/home.vue" },
  { name: "user", path: "/users/:id/:tab?", component: "./routes/user.vue" },
  { name: "docs", path: "/docs/*slug", component: "./routes/docs.vue" },
] as const);

function serverState(input = "/users/42?tag=a&tag=b#details") {
  const match = createRouterMatcher(routes).match(input);
  assert.ok(match);
  return serializeRouteMatch(match, { loaded: true });
}

void test("hydrates JSON-round-tripped state against the activation URL without browser globals", () => {
  for (const url of [
    "/users/42?tag=a&tag=b#details",
    "/users/a%2Fb/settings",
    "/docs/guide/router",
  ]) {
    const state = JSON.parse(JSON.stringify(serverState(url)));
    assert.deepEqual(
      hydrateRouteState(routes, state, new URL(url, "https://example.test")),
      createRouterMatcher(routes).match(url),
    );
  }
});

void test("rejects stale paths, names and params instead of trusting the serialized match", () => {
  const state = serverState();
  for (const change of [
    { name: "home" },
    { name: "deleted" },
    { pathname: "/missing" },
    { pathname: "/users/43" },
    { params: {} },
    { params: { id: "43" } },
    { params: { id: "42", extra: "unexpected" } },
    { pathname: "/users/42?query=hidden" },
    { pathname: "/users/42#hidden" },
    { pathname: "https://example.test/users/42" },
    { pathname: "//example.test/users/42" },
    { pathname: "/users/%E0%A4%A" },
  ]) {
    assert.equal(
      hydrateRouteState(routes, { ...state, ...change } as never),
      undefined,
      JSON.stringify(change),
    );
  }
});

void test("rejects malformed serialized payloads without throwing", () => {
  const state = serverState();
  for (const invalid of [
    null,
    {},
    { ...state, schemaVersion: 2 },
    { ...state, pathname: 42 },
    { ...state, params: null },
    { ...state, params: [] },
    { ...state, params: { id: 42 } },
    { ...state, query: null },
    { ...state, query: [] },
    { ...state, query: { tag: ["a", 1] } },
    { ...state, hash: null },
  ]) {
    assert.equal(hydrateRouteState(routes, invalid as never), undefined);
  }
});

void test("client activation refuses stale URLs and preserves repeated query order", () => {
  const state = serverState();
  for (const input of [
    "/",
    "/users/43?tag=a&tag=b#details",
    "/users/42?tag=b&tag=a#details",
    "/users/42?tag=a#details",
    "/users/42?tag=a&tag=b#other",
    "/users/%E0%A4%A",
    "http://[",
  ]) {
    assert.equal(hydrateRouteState(routes, state, input), undefined, input);
  }
  const reordered = serverState("/users/42?a=1&b=2");
  assert.ok(hydrateRouteState(routes, reordered, "/users/42?b=2&a=1"));
});

void test("route table changes invalidate the old handoff and fresh server state repairs it", () => {
  const changed = defineRoutes([
    { name: "user", path: "/people/:id", component: "./routes/user.vue", meta: { revision: 2 } },
  ] as const);
  assert.equal(hydrateRouteState(changed, serverState() as never), undefined);
  const match = createRouterMatcher(changed).match("/people/42");
  assert.ok(match);
  assert.deepEqual(hydrateRouteState(changed, serializeRouteMatch(match), "/people/42"), match);
});

void test("prototype-shaped query names remain ordinary own query values", () => {
  const input = "/users/42?__proto__=a&__proto__=b&constructor=c&toString=d";
  const state = JSON.parse(JSON.stringify(serverState(input)));
  assert.deepEqual(
    state.query,
    JSON.parse('{"__proto__":["a","b"],"constructor":"c","toString":"d"}'),
  );
  assert.deepEqual(hydrateRouteState(routes, state, input)?.query, state.query);
  assert.equal(Object.getPrototypeOf(state.query), Object.prototype);
});
