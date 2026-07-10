import { test } from "node:test";
import { createFilter } from "./utils/filter.ts";

void test("string glob include matches absolute Vue SFC paths", (t) => {
  const filter = createFilter(["**/*.vue", "../design/**/*.vue"]);

  t.assert.equal(filter("/repo/app/layouts/header-only.vue"), true);
  t.assert.equal(filter("/repo/design/components/Button.vue"), true);
  t.assert.equal(filter("/repo/app/routes/home.ts"), false);
});

void test("string glob exclude matches absolute generated directories", (t) => {
  const filter = createFilter("**/*.vue", ["../node_modules/**", "../.nuxt/**", "../.output/**"]);

  t.assert.equal(filter("/repo/app/pages/index.vue"), true);
  t.assert.equal(filter("/repo/node_modules/pkg/Widget.vue"), false);
  t.assert.equal(filter("/repo/app/.nuxt/components/App.vue"), false);
  t.assert.equal(filter("/repo/app/.output/server/App.vue"), false);
});

void test("plain string filters keep substring semantics", (t) => {
  const filter = createFilter(".custom", ".generated.");

  t.assert.equal(filter("/repo/app/x.custom"), true);
  t.assert.equal(filter("/repo/app/x.generated.custom"), false);
});

void test("global regex filters are stable across repeated calls", (t) => {
  const filter = createFilter(/\.vue$/g, /node_modules/g);

  t.assert.equal(filter("/repo/app/App.vue"), true);
  t.assert.equal(filter("/repo/app/App.vue"), true);
  t.assert.equal(filter("/repo/node_modules/pkg/App.vue"), false);
  t.assert.equal(filter("/repo/node_modules/pkg/App.vue"), false);
});
