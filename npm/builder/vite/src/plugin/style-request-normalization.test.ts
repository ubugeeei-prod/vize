import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import {
  createStyleVirtualId,
  normalizeStyleVirtualId,
  resolveStyleVirtualId,
} from "./style-request.ts";

// The fresh native source fixture returned HTTP 404 for this actual request.
// Vite prepends `direct` to browser stylesheet requests before plugin loading.
const actual =
  "/VFHeading.vue.__vize_style_0.css?direct&vue=&type=style&index=0&scoped=data-v-ea49ded1&lang=css&vize-file=%2Fhome%2Frunner%2F_work%2Fvize%2Fvize%2Ftarget%2Fvuefes-ssr-style-dependencies%2Fproject-1Rp9Ub%2FVFHeading%2Evue";
const source =
  "/home/runner/_work/vize/vize/target/vuefes-ssr-style-dependencies/project-1Rp9Ub/VFHeading.vue";

void test("real browser direct CSS requests retain the authored source and entire query", () => {
  for (const id of [actual, `\0${actual}`, actual.replace("?direct&vue", "?vue")]) {
    assert.equal(normalizeStyleVirtualId(id), `${source}?${id.slice(id.indexOf("?") + 1)}`);
  }
});

void test("lookalike query names cannot opt into Vue style normalization", () => {
  for (const id of [
    actual.replace("&vue=", "&vueish="),
    actual.replace("&vue=", "&notvue="),
    actual.replace("&vue=", "&x=vue&"),
    "/VFHeading.vue.__vize_style_0.css",
  ])
    assert.equal(normalizeStyleVirtualId(id), id);
});

void test("stable production imports resolve with a physical CSS origin and unchanged metadata", () => {
  const query =
    "vue=&type=style&index=2&scoped=data-v-abcd1234&lang=scss&module=theme%2Emodule&vize-root-relative=&vize-file=%2Fsrc%2Fcomponents%2FCard%20%E9%9B%AA%2Evue&inline&used";
  const stable = `/src/components/Card 雪.vue.__vize_style_2.module.scss?${query}`;
  for (const root of [
    path.resolve("checkout a", "app"),
    path.resolve("checkout b", "nested", "app"),
  ]) {
    const physical = path.join(root, "src/components/Card 雪.vue").split(path.sep).join("/");
    const expected = `${physical}.__vize_style_2.module.scss?${query}`;
    assert.equal(createStyleVirtualId(stable), stable);
    assert.equal(resolveStyleVirtualId(stable, root), expected);
    assert.equal(
      resolveStyleVirtualId(expected, root),
      expected,
      "replayed resolution is idempotent",
    );
    assert.equal(
      createStyleVirtualId(expected),
      stable,
      "the producer identity remains checkout independent",
    );
    assert.equal(normalizeStyleVirtualId(expected), `/src/components/Card 雪.vue?${query}`);
  }
});

void test("absolute Nuxt and outside-root styles retain their original physical origin", () => {
  const root = path.resolve("project", "app");
  const source = path.resolve("shared", "Outside.vue").split(path.sep).join("/");
  const query = `vue=&type=style&index=0&lang=css&vize-file=${encodeURIComponent(source).replaceAll(".", "%2E")}`;
  const expected = `${source}.__vize_style_0.css?${query}`;
  assert.equal(
    resolveStyleVirtualId(`shared/Outside.vue.__vize_style_0.css?${query}`, root),
    expected,
  );
  assert.equal(resolveStyleVirtualId(expected, root), expected);
  const lookalike = "/src/Card.vue?vue=&type=style&index=0&lang=css&not-vize-root-relative=";
  assert.equal(resolveStyleVirtualId(lookalike, root), createStyleVirtualId(lookalike));
});

void test("root-relative dependency styles keep their package directory as the CSS origin", () => {
  const root = path.resolve("project", "app");
  const source = "/node_modules/relative-card/src/Card.vue";
  const query =
    "vue=&type=style&index=0&lang=css&vize-root-relative=&vize-file=%2Fnode_modules%2Frelative-card%2Fsrc%2FCard%2Evue";
  const stable = `${source}.__vize_style_0.css?${query}`;
  assert.equal(resolveStyleVirtualId(stable, root), `${root.split(path.sep).join("/")}${stable}`);
  assert.equal(createStyleVirtualId(stable), stable);
});
