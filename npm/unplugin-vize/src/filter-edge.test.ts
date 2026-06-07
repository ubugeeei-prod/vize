import { test } from "node:test";
import { createFilter } from "./filter.ts";

void test("defaults: include .vue, exclude node_modules, ignore non-.vue", (t) => {
  const filter = createFilter();

  // Matches default include (/\.vue$/), not in node_modules.
  t.assert.equal(filter("/proj/src/App.vue"), true);
  // Matches default include but hits default exclude (/node_modules/).
  t.assert.equal(filter("/proj/node_modules/x/App.vue"), false);
  // Does not match default include (.ts is not .vue).
  t.assert.equal(filter("/proj/src/App.ts"), false);
});

void test("string include uses substring inclusion", (t) => {
  const filter = createFilter(".custom");

  t.assert.equal(filter("/a/x.custom"), true);
  t.assert.equal(filter("/a/x.other"), false);
});

void test("array include with mixed string + regex matches either", (t) => {
  const filter = createFilter([/\.vue$/, ".md"]);

  t.assert.equal(filter("/a.vue"), true);
  t.assert.equal(filter("/docs/x.md"), true);
  // Matches neither pattern.
  t.assert.equal(filter("/a.ts"), false);
});

void test("custom include REPLACES the default .vue$ include", (t) => {
  // A custom include that does not cover .vue makes .vue files NOT match.
  const filter = createFilter(".md");

  t.assert.equal(filter("/a/x.md"), true);
  t.assert.equal(filter("/a/App.vue"), false);
});

void test("custom exclude REPLACES the default node_modules exclude", (t) => {
  // With a custom exclude that doesn't hit node_modules, a node_modules path
  // that matches include is now INCLUDED.
  const filter = createFilter(/\.vue$/, ".cache");

  // node_modules path now passes because default node_modules exclusion is gone.
  t.assert.equal(filter("/proj/node_modules/x/App.vue"), true);
  // The custom exclude (".cache") still excludes matching paths.
  t.assert.equal(filter("/proj/.cache/App.vue"), false);
});

void test("array exclude: any matching exclude pattern wins", (t) => {
  const filter = createFilter(/\.vue$/, [/node_modules/, ".generated."]);

  t.assert.equal(filter("/proj/src/App.vue"), true);
  t.assert.equal(filter("/proj/node_modules/x/App.vue"), false);
  t.assert.equal(filter("/proj/src/App.generated.vue"), false);
});

void test("include matches but exclude also matches => excluded", (t) => {
  // include matches /\.vue$/ but exclude substring "secret" also matches.
  const filter = createFilter(/\.vue$/, "secret");

  t.assert.equal(filter("/proj/secret/App.vue"), false);
  t.assert.equal(filter("/proj/public/App.vue"), true);
});

void test("string-vs-regex precedence: both kinds honored together", (t) => {
  // string include (substring) plus regex exclude (anchored).
  const filter = createFilter("src/", /\.spec\.vue$/);

  // In src/ and not a spec file.
  t.assert.equal(filter("/proj/src/App.vue"), true);
  // In src/ but excluded by regex.
  t.assert.equal(filter("/proj/src/App.spec.vue"), false);
  // Not in src/ at all => not included.
  t.assert.equal(filter("/proj/lib/App.vue"), false);
});

void test("empty array include matches nothing", (t) => {
  // [].some(...) is false, so include never matches.
  const filter = createFilter([]);

  t.assert.equal(filter("/a/App.vue"), false);
  t.assert.equal(filter("/a/x.custom"), false);
});

void test("empty array exclude excludes nothing", (t) => {
  // [].some(...) is false, so default node_modules guard is gone.
  const filter = createFilter(/\.vue$/, []);

  t.assert.equal(filter("/proj/node_modules/x/App.vue"), true);
  t.assert.equal(filter("/proj/src/App.vue"), true);
});
