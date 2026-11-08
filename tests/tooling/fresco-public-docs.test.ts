import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const read = (relativePath: string) =>
  readFileSync(new URL(`../../${relativePath}`, import.meta.url), "utf8");

void test("Fresco migration guides use only supported public entrypoints", () => {
  const ink = read("docs/fresco/migration-ink.md");
  const vueTermUi = read("docs/fresco/migration-vue-termui.md");

  for (const source of [ink, vueTermUi]) {
    assert.match(source, /from "@vizejs\/fresco"/u);
    assert.doesNotMatch(source, /@vizejs\/fresco\/src\//u);
    assert.match(source, /\.\/compatibility\.md/u);
  }
  assert.match(vueTermUi, /npm\/fresco\/examples\/sfc-app/u);
});

void test("Fresco publishes explicit pre-1.0 and extension compatibility rules", () => {
  const policy = read("docs/fresco/support-policy.md");
  const matrix = read("docs/fresco/compatibility.md");

  assert.match(policy, /Patch releases do not intentionally break/u);
  assert.match(policy, /Minor releases may change an incubating public API/u);
  assert.match(policy, /Mixing versions is unsupported/u);
  assert.match(policy, /FrameBuffer/u);
  assert.match(policy, /Imports from `src\/`/u);
  assert.match(matrix, /\.\/migration-ink\.md/u);
  assert.match(matrix, /\.\/migration-vue-termui\.md/u);
  assert.match(matrix, /\.\/support-policy\.md/u);
});
