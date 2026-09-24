import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { oneOf, queryParsers, useRouteParams, useRouteQuery } from "./use-route-query.ts";
import type { RouteQueryRecord } from "./use-route-query.ts";

interface Match {
  readonly name: "user";
  readonly params: { readonly id: string; readonly tab?: string };
  readonly query: RouteQueryRecord;
}

function createRoute(query: RouteQueryRecord = {}) {
  const route = shallowRef<Match>({ name: "user", params: { id: "42" }, query });
  const pushed: RouteQueryRecord[] = [];
  const navigate = (next: RouteQueryRecord): void => {
    pushed.push(next);
    route.value = { ...route.value, query: next };
  };
  return { route, pushed, navigate };
}

void test("reads and parses query values with defaults", () => {
  const { route } = createRoute({ page: "3", tags: ["a", "b"], debug: "", sort: "top" });
  assert.equal(useRouteQuery(route, "page", { parse: queryParsers.integer, default: 1 }).value, 3);
  assert.deepEqual(useRouteQuery(route, "tags", { parse: queryParsers.array }).value, ["a", "b"]);
  assert.equal(useRouteQuery(route, "debug", { parse: queryParsers.boolean }).value, true);
  assert.equal(
    useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "new" }).value,
    "top",
  );
  assert.equal(useRouteQuery(route, "missing").value, undefined);
  assert.equal(useRouteQuery(route, "tags").value, "a");

  route.value = { ...route.value, query: { page: "x", sort: "old" } };
  assert.equal(useRouteQuery(route, "page", { parse: queryParsers.integer, default: 1 }).value, 1);
  assert.equal(
    useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "new" }).value,
    "new",
  );
});

void test("built-in parsers reject malformed values", () => {
  assert.equal(queryParsers.number(" "), undefined);
  assert.equal(queryParsers.number("1e3"), 1000);
  assert.equal(queryParsers.number("Infinity"), undefined);
  assert.equal(queryParsers.integer("1.5"), undefined);
  assert.equal(queryParsers.integer("99999999999999999999"), undefined);
  assert.equal(queryParsers.boolean("0"), false);
  assert.equal(queryParsers.boolean("maybe"), undefined);
  assert.equal(queryParsers.string(["x", "y"]), "x");
  assert.equal(queryParsers.array(undefined), undefined);
});

void test("writes keep other keys, drop defaults, and skip no-op navigations", () => {
  const { route, pushed, navigate } = createRoute({ page: "2", q: "vize" });
  const page = useRouteQuery(route, "page", {
    parse: queryParsers.integer,
    default: 1,
    navigate,
  });
  page.value = 5;
  assert.deepEqual(pushed.at(-1), { page: "5", q: "vize" });
  assert.equal(page.value, 5);
  page.value = 1;
  assert.deepEqual(pushed.at(-1), { q: "vize" });
  page.value = 1;
  assert.equal(pushed.length, 2, "writing the current value does not navigate");

  const tags = useRouteQuery(route, "tags", { parse: queryParsers.array, navigate });
  tags.value = ["a", "b"];
  assert.deepEqual(pushed.at(-1), { q: "vize", tags: ["a", "b"] });
  tags.value = undefined;
  assert.deepEqual(pushed.at(-1), { q: "vize" });
});

void test("custom serializers and keeping defaults", () => {
  const { route, pushed, navigate } = createRoute();
  const range = useRouteQuery(route, "range", {
    parse: (raw) => {
      const text = queryParsers.string(raw);
      const parts = text?.split("-").map(Number);
      return parts?.length === 2 ? { from: parts[0] ?? 0, to: parts[1] ?? 0 } : undefined;
    },
    serialize: (value) => `${value.from}-${value.to}`,
    default: { from: 0, to: 10 },
    omitDefault: false,
    navigate,
  });
  assert.deepEqual(range.value, { from: 0, to: 10 });
  range.value = { from: 0, to: 10 };
  assert.deepEqual(pushed.at(-1), { range: "0-10" });
  range.value = { from: 5, to: 7 };
  assert.deepEqual(route.value.query, { range: "5-7" });
});

void test("route params, single params, and parsed params", () => {
  const { route } = createRoute();
  assert.deepEqual(useRouteParams(route).value, { id: "42" });
  assert.equal(useRouteParams(route, "id").value, "42");
  const id = useRouteParams(route, "id", Number);
  assert.equal(id.value, 42);
  route.value = { ...route.value, params: { id: "7", tab: "posts" } };
  assert.equal(id.value, 7);
  assert.equal(useRouteParams(route, "tab").value, "posts");
});

void test("renders the matched route deterministically on the server", async () => {
  const state = await renderComposableOnServer(() => {
    const { route, navigate } = createRoute({ page: "4", sort: "top" });
    return {
      page: useRouteQuery(route, "page", { parse: queryParsers.integer, default: 1, navigate }),
      sort: useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "new" }),
      id: useRouteParams(route, "id", Number),
    };
  });
  assert.equal(state, '{"page":4,"sort":"top","id":42}');
});
