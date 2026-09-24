import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import {
  parseSearchParams,
  searchParam,
  serializeSearchParams,
  useUrlSearchParams,
} from "./use-url-search-params.ts";
import type { UrlSearchParamsHost } from "./use-url-search-params.ts";

class FakeWindow extends EventTarget implements UrlSearchParamsHost {
  readonly location = { search: "", hash: "#top", pathname: "/list" };
  readonly calls: { readonly kind: "push" | "replace"; readonly url: string }[] = [];
  listeners = 0;
  readonly history = {
    state: null as unknown,
    replaceState: (_data: unknown, _unused: string, url: string): void => {
      this.calls.push({ kind: "replace", url });
      this.setUrl(url);
    },
    pushState: (_data: unknown, _unused: string, url: string): void => {
      this.calls.push({ kind: "push", url });
      this.setUrl(url);
    },
  };

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  setUrl(url: string): void {
    const withoutHash = url.split("#", 1)[0] ?? "";
    const query = withoutHash.indexOf("?");
    this.location.search = query === -1 ? "" : withoutHash.slice(query);
  }

  navigate(search: string): void {
    this.location.search = search;
    this.dispatchEvent(new Event("popstate"));
  }
}

const schema = {
  q: searchParam.string(),
  page: searchParam.number(1),
  open: searchParam.boolean(),
  sort: searchParam.enum(["new", "top"], "new"),
  tags: searchParam.array(),
  ids: searchParam.array(searchParam.number()),
  range: searchParam.json({ from: 0, to: 10 }),
  day: searchParam.custom({
    parse: (raw) => new Date(`${raw}T00:00:00.000Z`),
    serialize: (value: Date) => value.toISOString().slice(0, 10),
    default: new Date(0),
  }),
};

void test("parses every codec kind with defaults and fallbacks", () => {
  const values = parseSearchParams(
    schema,
    "https://example.test/list?q=vue&page=x&open&sort=top&tags=a&tags=b&ids=1&ids=2&range=%7B%22from%22%3A2%2C%22to%22%3A3%7D&day=2026-09-25#h",
  );
  assert.equal(values.q, "vue");
  assert.equal(values.page, 1, "invalid numbers fall back");
  assert.equal(values.open, true);
  assert.equal(values.sort, "top");
  assert.deepEqual(values.tags, ["a", "b"]);
  assert.deepEqual(values.ids, [1, 2]);
  assert.deepEqual(values.range, { from: 2, to: 3 });
  assert.equal(values.day.toISOString(), "2026-09-25T00:00:00.000Z");

  const defaults = parseSearchParams(schema, "?sort=bogus&range=%5B%5D&range2");
  assert.equal(defaults.sort, "new");
  assert.deepEqual(defaults.range, { from: 0, to: 10 }, "wrong JSON kind falls back");
  assert.deepEqual(defaults.tags, []);
  assert.deepEqual(parseSearchParams(schema, "?range={broken").range, { from: 0, to: 10 });
  assert.deepEqual(parseSearchParams(schema, new URLSearchParams("q=1")).q, "1");
  assert.deepEqual(parseSearchParams(schema, new URL("https://x.test/?page=3")).page, 3);
});

void test("serializes values, keeps foreign keys, and drops defaults", () => {
  const params = serializeSearchParams(
    schema,
    { page: 1, sort: "top", tags: ["x", "y"], q: "" },
    { base: "?utm=1&page=4" },
  );
  assert.equal(params.toString(), "utm=1&sort=top&tags=x&tags=y");

  const kept = serializeSearchParams(schema, { page: 1 }, { removeDefaults: false });
  assert.equal(kept.toString(), "page=1");
});

void test("binds typed params to the URL through replaceState", () => {
  const host = new FakeWindow();
  host.location.search = "?page=2&utm=mail";
  const { params, supported } = useUrlSearchParams(schema, { host });

  assert.equal(supported.value, true);
  assert.equal(params.page, 2);
  assert.equal(host.calls.length, 0, "reading does not write");

  params.page = 3;
  assert.deepEqual(host.calls.at(-1), { kind: "replace", url: "/list?page=3&utm=mail#top" });
  params.tags = ["a"];
  assert.deepEqual(host.calls.at(-1), {
    kind: "replace",
    url: "/list?page=3&utm=mail&tags=a#top",
  });
  params.page = 1;
  assert.deepEqual(host.calls.at(-1), { kind: "replace", url: "/list?utm=mail&tags=a#top" });
});

void test("pushes entries in push mode and re-reads on popstate without echo", () => {
  const host = new FakeWindow();
  const { params } = useUrlSearchParams({ q: searchParam.string() }, { host, mode: "push" });

  params.q = "a";
  assert.deepEqual(host.calls, [{ kind: "push", url: "/list?q=a#top" }]);
  host.navigate("?q=b&extra=1");
  assert.equal(params.q, "b");
  assert.equal(host.calls.length, 1);
  host.navigate("");
  assert.equal(params.q, "");
  assert.equal(host.calls.length, 1);
});

void test("stops following the URL with the scope", () => {
  const host = new FakeWindow();
  const scope = effectScope();
  const controls = scope.run(() => useUrlSearchParams({ q: searchParam.string() }, { host }));
  assert.ok(controls);
  assert.equal(host.listeners, 1);
  scope.stop();
  assert.equal(host.listeners, 0);
  controls.params.q = "late";
  assert.equal(host.calls.length, 0);
});

void test("server rendering reads the request URL", async () => {
  const state = await renderComposableOnServer(() => {
    const { params, supported } = useUrlSearchParams(
      { page: searchParam.number(1), tags: searchParam.array() },
      { ssrUrl: "/list?page=5&tags=a&tags=b" },
    );
    return { page: params.page, tags: params.tags, supported };
  });
  assert.equal(state, '{"page":5,"tags":["a","b"],"supported":false}');
});

void test("server rendering without a request URL uses codec defaults", async () => {
  const state = await renderComposableOnServer(() => {
    const { params } = useUrlSearchParams({ page: searchParam.number(1) });
    return { page: params.page };
  });
  assert.equal(state, '{"page":1}');
});
