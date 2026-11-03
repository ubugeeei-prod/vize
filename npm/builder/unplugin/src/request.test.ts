import { test, type TestContext } from "node:test";
import { isVueFile, isVueStyleRequest, parseVueRequest, styleRequestId } from "./request.ts";
import type { StyleBlockInfo } from "./types.ts";

void test("isVueFile matches only ids ending in .vue (endsWith semantics)", (t) => {
  t.assert.equal(isVueFile("/a/App.vue"), true);
  // A style sub-request does NOT end in `.vue`, so endsWith bails.
  t.assert.equal(isVueFile("/a/App.vue?vue&type=style"), false);
  // endsWith is case-sensitive: uppercase extension is not a match.
  t.assert.equal(isVueFile("/a/app.VUE"), false);
  t.assert.equal(isVueFile(""), false);
});

void test("isVueStyleRequest fast-path bails without a ?vue marker", (t) => {
  // No `?vue` substring -> short-circuits before any URLSearchParams parsing.
  t.assert.equal(isVueStyleRequest("/a/App.vue"), false);
  t.assert.equal(isVueStyleRequest("/a/App.vue?type=style"), false);
});

void test("isVueStyleRequest is true only for vue + type=style", (t) => {
  t.assert.equal(isVueStyleRequest("/a/App.vue?vue&type=style"), true);
  t.assert.equal(isVueStyleRequest("/a/App.vue?vue&type=style&index=0"), true);
  // type=template carries `?vue` (passes fast-path) but is not a style request.
  t.assert.equal(isVueStyleRequest("/a/App.vue?vue&type=template"), false);
});

void test("parseVueRequest splits on the first ? only", (t) => {
  // Extra `?`, `=`, `&` after the first `?` stay inside the raw query string.
  const parsed = parseVueRequest("/a/App.vue?vue&type=style&a=b?c=d");
  t.assert.equal(parsed.filename, "/a/App.vue");
  t.assert.equal(parsed.query.vue, true);
  t.assert.equal(parsed.query.type, "style");
});

void test("parseVueRequest module param: present-empty -> true, string -> value, absent -> false", (t) => {
  // `?module` (no value) -> true.
  t.assert.equal(parseVueRequest("/a/App.vue?module").query.module, true);
  // `?module=foo` -> the string value.
  t.assert.equal(parseVueRequest("/a/App.vue?module=foo").query.module, "foo");
  // absent -> false.
  t.assert.equal(parseVueRequest("/a/App.vue?vue").query.module, false);
});

void test("parseVueRequest index parsing covers numbers, NaN, and absence", (t) => {
  t.assert.equal(parseVueRequest("/a/App.vue?index=0").query.index, 0);
  t.assert.equal(parseVueRequest("/a/App.vue?index=3").query.index, 3);
  t.assert.equal(parseVueRequest("/a/App.vue?index=-1").query.index, -1);
  // Non-numeric index parses to NaN (Number.parseInt), NOT null.
  t.assert.equal(Number.isNaN(parseVueRequest("/a/App.vue?index=abc").query.index), true);
  // Absent index is null.
  t.assert.equal(parseVueRequest("/a/App.vue?vue").query.index, null);
});

void test("parseVueRequest leaves type/scoped null when absent", (t) => {
  const q = parseVueRequest("/a/App.vue").query;
  t.assert.equal(q.type, null);
  t.assert.equal(q.scoped, null);
});

function block(overrides: Partial<StyleBlockInfo>): StyleBlockInfo {
  return {
    content: "",
    src: null,
    lang: null,
    scoped: false,
    module: false,
    index: 0,
    ...overrides,
  };
}

// Hosts classify a stylesheet by the extension that ends the id (Rolldown,
// tsdown's CSS guard, rollup-plugin-postcss) and skip a stylesheet path that
// carries a query (@tsdown/css), so the id ends in `.<lang>` on the SFC path.
function assertHostShape(t: TestContext, id: string, ext: string) {
  t.assert.equal(id.split("?", 1)[0], "/a/App.vue");
  t.assert.equal(id.endsWith(`&lang.${ext}`), true, id);
  t.assert.equal(isVueStyleRequest(id), true);
}

void test("styleRequestId names a plain block by its language", (t) => {
  const id = styleRequestId("/a/App.vue", block({ lang: "less", index: 2 }), null, "preprocessors");
  t.assert.equal(id, "/a/App.vue?vue&type=style&index=2&lang.less");
  assertHostShape(t, id, "less");
  t.assert.equal(
    styleRequestId("/a/App.vue", block({}), null, "preprocessors"),
    "/a/App.vue?vue&type=style&index=0&lang.css",
  );
});

void test("styleRequestId names a scoped preprocessor block as CSS", (t) => {
  // load() preprocesses and then scopes it, so bundlers receive plain CSS.
  const id = styleRequestId(
    "/a/App.vue",
    block({ lang: "scss", scoped: true, index: 1 }),
    "data-v-z",
    "preprocessors",
  );
  t.assert.equal(id, "/a/App.vue?vue&type=style&index=1&scoped=data-v-z&lang.css");
  assertHostShape(t, id, "css");
  t.assert.equal(parseVueRequest(id).query.scoped, "data-v-z");
});

void test("styleRequestId marks CSS modules in the query and the extension", (t) => {
  const unnamed = styleRequestId(
    "/a/App.vue",
    block({ module: true, lang: "scss" }),
    null,
    "preprocessors",
  );
  t.assert.equal(unnamed, "/a/App.vue?vue&type=style&index=0&module&lang.module.scss");
  assertHostShape(t, unnamed, "module.scss");
  t.assert.equal(parseVueRequest(unnamed).query.module, true);

  const named = styleRequestId(
    "/a/App.vue",
    block({ module: "my mod", index: 3 }),
    "data-v-r",
    "preprocessors",
  );
  t.assert.equal(
    named,
    "/a/App.vue?vue&type=style&index=3&scoped=data-v-r&module=my%20mod&lang.module.css",
  );
  assertHostShape(t, named, "module.css");
  const back = parseVueRequest(named);
  t.assert.equal(back.filename, "/a/App.vue");
  t.assert.equal(back.query.vue, true);
  t.assert.equal(back.query.type, "style");
  t.assert.equal(back.query.index, 3);
  t.assert.equal(back.query.module, "my mod");
  t.assert.equal(back.query.lang, "css");
});

void test("styleRequestId names a preprocessor block as CSS for a host that compiles none", (t) => {
  // #11513: esbuild's CSS loaders compile no Sass, so load() compiles it.
  const scss = block({ lang: "scss" });
  t.assert.equal(
    styleRequestId("/a/App.vue", scss, null, "css"),
    "/a/App.vue?vue&type=style&index=0&lang.css",
  );
  t.assert.equal(
    styleRequestId("/a/App.vue", { ...scss, module: true }, null, "css"),
    "/a/App.vue?vue&type=style&index=0&module&lang.module.css",
  );
  // Negative control: a host whose plugins compile Sass gets the source.
  const raw = styleRequestId("/a/App.vue", scss, null, "preprocessors");
  t.assert.equal(raw, "/a/App.vue?vue&type=style&index=0&lang.scss");
  t.assert.equal(parseVueRequest(raw).query.lang, "scss");
  t.assert.equal(parseVueRequest("/a/App.vue?vue&type=style").query.lang, null);
});
