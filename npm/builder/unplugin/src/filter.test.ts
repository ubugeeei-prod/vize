import { test } from "node:test";
import { createFilter } from "./filter.ts";

void test("global include regex stays stable across repeated calls", (t) => {
  const filter = createFilter(/\.vue$/g);

  t.assert.equal(filter("/project/src/App.vue"), true);
  t.assert.equal(filter("/project/src/App.vue"), true);
});

void test("global exclude regex stays stable across repeated calls", (t) => {
  const filter = createFilter(undefined, /node_modules/g);

  t.assert.equal(filter("/project/node_modules/pkg/App.vue"), false);
  t.assert.equal(filter("/project/node_modules/pkg/App.vue"), false);
});

void test("regex matcher does not leak lastIndex back to callers", (t) => {
  const include = /\.vue$/g;
  const filter = createFilter(include);

  t.assert.equal(filter("/project/src/App.vue"), true);
  t.assert.equal(include.lastIndex, 0);
});
