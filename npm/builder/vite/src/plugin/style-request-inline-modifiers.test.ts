import assert from "node:assert/strict";
import { test } from "node:test";
import { createStyleVirtualId, resolveStyleVirtualId } from "./style-request.ts";

// Nuxt 3.19.3's SSRStylesPlugin appends this literal suffix to an ID that
// already has query metadata. This is the failed error-404.vue request shape.
const source = "/node_modules/nuxt/dist/app/components/error-404.vue";
const sourceQuery = "%2Fnode_modules%2Fnuxt%2Fdist%2Fapp%2Fcomponents%2Ferror-404%2Evue";
const runnerRoot =
  "/home/runner/_work/vize/vize/tools/support/compat/nuxt/fixtures/nuxt3-module-build";

void test("Nuxt inline CSS modifiers preserve physical origins and block metadata", () => {
  for (const root of [runnerRoot, "/checkout/日本語 project"]) {
    for (const [extension, metadata] of [
      ["css", "scoped=data-v-b03b9ec4&lang=css"],
      ["css", "lang=css"],
      ["module.scss", "scoped=data-v-b03b9ec4&lang=scss&module=theme"],
    ]) {
      const query = `vue=&type=style&index=0&${metadata}&vize-root-relative=&vize-file=${sourceQuery}`;
      const physical = `${root}${source}.__vize_style_0.${extension}?${query}`;
      const expected = `${physical}&inline&used`;
      const actual = resolveStyleVirtualId(`${physical}?inline&used`, root);
      assert.equal(actual, expected);
      assert.equal(resolveStyleVirtualId(actual, root), expected);
      const params = new URLSearchParams(actual.slice(actual.indexOf("?") + 1));
      assert.equal(params.get("vize-file"), source);
      assert.deepEqual(params.getAll("inline"), [""]);
      assert.deepEqual(params.getAll("used"), [""]);
      assert.equal(params.get("type"), "style");
      assert.equal(params.get("index"), "0");
      assert.equal(params.get("module"), extension === "module.scss" ? "theme" : null);
    }
  }
});

void test("producer IDs retain inline modifiers when reconstructed before physical resolution", () => {
  const query = `vue=&type=style&index=0&lang=css&vize-root-relative=&vize-file=${sourceQuery}`;
  const emitted = `${source}.__vize_style_0.css?${query}`;
  assert.equal(createStyleVirtualId(`${emitted}?inline&used`), `${emitted}&inline&used`);
  assert.equal(
    resolveStyleVirtualId(`${emitted}?inline&used`, runnerRoot),
    `${runnerRoot}${emitted}&inline&used`,
  );
});

void test("legitimate encoded and literal query values keep their exact bytes", () => {
  const query = `vue=&type=style&index=0&lang=css&vize-root-relative=&vize-file=${sourceQuery}`;
  const physical = `${runnerRoot}${source}.__vize_style_0.css?${query}`;
  for (const suffix of [
    "&token=%3Finline%26used",
    "&token=?inline&used",
    "&token=before%3Finline%26usedafter",
  ]) {
    assert.equal(resolveStyleVirtualId(physical + suffix, runnerRoot), physical + suffix);
  }
});

void test("unknown appended transports stay CSS requests without inventing a Vue query", () => {
  const query = `vue=&type=style&index=0&lang=css&vize-root-relative=&vize-file=${sourceQuery}`;
  const physical = `${runnerRoot}${source}.__vize_style_0.css?${query}`;
  for (const suffix of ["?unknown&used", "?inline", "?inline&used=other", "?inline&used&other"]) {
    assert.equal(resolveStyleVirtualId(physical + suffix, runnerRoot), physical + suffix);
  }
});

void test("invalid marker and duplicate reserved metadata fail conservatively", () => {
  const prefix = `${runnerRoot}${source}.__vize_style_0.css?vue=&type=style&index=0&lang=css&vize-root-relative=`;
  for (const id of [
    `${prefix}&vize-file=${sourceQuery}&vize-file=${sourceQuery}?inline&used`,
    `${prefix}&type=script&vize-file=${sourceQuery}?inline&used`,
    `${prefix}&type=style&vize-file=${sourceQuery}?inline&used`,
    `${prefix}&vize-file=${sourceQuery}%3Flegitimate?inline&used`,
    `${runnerRoot}${source}?vue=&type=style&vize-root-relative=&vize-file=${sourceQuery}?inline&used`,
  ]) {
    assert.equal(resolveStyleVirtualId(id, runnerRoot), id);
  }
});
