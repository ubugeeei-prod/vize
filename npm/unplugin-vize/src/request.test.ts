import { test } from "node:test";
import {
  createVirtualStyleId,
  isVirtualStyleId,
  isVueFile,
  isVueStyleRequest,
  parseVueRequest,
} from "./request.ts";

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

void test("isVirtualStyleId detects the STYLE_MARKER substring", (t) => {
  const vid = createVirtualStyleId("/a/App.vue?vue&type=style");
  t.assert.equal(isVirtualStyleId(vid), true);
  t.assert.equal(isVirtualStyleId("/a/App.vue"), false);
});

void test("parseVueRequest splits on the first ? only", (t) => {
  // Extra `?`, `=`, `&` after the first `?` stay inside the raw query string.
  const parsed = parseVueRequest("/a/App.vue?vue&type=style&a=b?c=d");
  t.assert.equal(parsed.path, "/a/App.vue");
  t.assert.equal(parsed.query.vue, true);
  t.assert.equal(parsed.query.type, "style");
});

void test("parseVueRequest lets vize-file override the path as filename", (t) => {
  const parsed = parseVueRequest("/a/App.vue?vue&type=style&vize-file=/real/Comp.vue");
  t.assert.equal(parsed.path, "/a/App.vue");
  t.assert.equal(parsed.filename, "/real/Comp.vue");
  // vizeFile query field mirrors the vize-file param.
  t.assert.equal(parsed.query.vizeFile, "/real/Comp.vue");
});

void test("parseVueRequest filename falls back to path when vize-file absent", (t) => {
  const parsed = parseVueRequest("/a/App.vue?vue");
  t.assert.equal(parsed.filename, "/a/App.vue");
  t.assert.equal(parsed.query.vizeFile, null);
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

void test("parseVueRequest leaves lang/type/scoped null when absent", (t) => {
  const q = parseVueRequest("/a/App.vue").query;
  t.assert.equal(q.lang, null);
  t.assert.equal(q.type, null);
  t.assert.equal(q.scoped, null);
});

void test("createVirtualStyleId emits the marker, index and .module.<lang> suffix", (t) => {
  const vid = createVirtualStyleId(
    "/a/App.vue?vue&type=style&index=1&lang=scss&module&scoped=data-v-z",
  );
  t.assert.equal(vid.includes(".__vize_style_1"), true);
  t.assert.equal(vid.includes(".module.scss?"), true);
  t.assert.equal(vid.startsWith("/a/App.vue.__vize_style_1.module.scss"), true);
});

void test("createVirtualStyleId uses .<lang> (no module segment) when module is false", (t) => {
  const vid = createVirtualStyleId("/a/App.vue?vue&type=style&lang=less");
  t.assert.equal(vid.includes(".module."), false);
  t.assert.equal(vid.includes(".__vize_style_0.less?"), true);
  // No `module` param is emitted when module === false.
  t.assert.equal(parseVueRequest(vid).query.module, false);
});

void test("createVirtualStyleId defaults index to 0 and lang to css", (t) => {
  const vid = createVirtualStyleId("/a/App.vue?vue&type=style");
  t.assert.equal(vid.includes(".__vize_style_0.css?"), true);
  const back = parseVueRequest(vid);
  t.assert.equal(back.query.index, 0);
  t.assert.equal(back.query.lang, "css");
});

void test("createVirtualStyleId preserves scoped and a named module", (t) => {
  const vid = createVirtualStyleId(
    "/a/App.vue?vue&type=style&index=2&lang=css&module=myMod&scoped=data-v-q",
  );
  const back = parseVueRequest(vid);
  t.assert.equal(back.query.scoped, "data-v-q");
  // A string module value round-trips as module=<name>.
  t.assert.equal(back.query.module, "myMod");
  // Named module still produces the .module.<lang> suffix.
  t.assert.equal(vid.includes(".module.css?"), true);
});

void test("createVirtualStyleId sets vize-file to the resolved filename", (t) => {
  const vid = createVirtualStyleId("/a/App.vue?vue&type=style&vize-file=/real/Comp.vue");
  const back = parseVueRequest(vid);
  t.assert.equal(back.query.vizeFile, "/real/Comp.vue");
  t.assert.equal(back.filename, "/real/Comp.vue");
  t.assert.equal(vid.startsWith("/real/Comp.vue.__vize_style_"), true);
});

void test("createVirtualStyleId output round-trips through parseVueRequest", (t) => {
  const vid = createVirtualStyleId(
    "/a/App.vue?vue&type=style&index=3&lang=scss&module=mod&scoped=data-v-r",
  );
  const back = parseVueRequest(vid);
  t.assert.equal(back.query.vue, true);
  t.assert.equal(back.query.type, "style");
  t.assert.equal(back.query.index, 3);
  t.assert.equal(back.query.lang, "scss");
  t.assert.equal(back.filename, "/a/App.vue");
});
