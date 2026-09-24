import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import {
  docsContentRoot,
  existingReferenceDocs,
  referencedComposables,
  renderReferenceDocs,
} from "../../npm/ui/scripts/generate-reference-docs.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";

await import("../../docs/theme/i18n/sitemap.js");
const sitemap = (
  globalThis as { __vizeDocsSitemap?: { navGroups: Array<{ key: string; paths: string[] }> } }
).__vizeDocsSitemap!;

const rendered = renderReferenceDocs();

void test("every UI catalog family has a reference page", () => {
  const missing = uiFamilyCatalog
    .map((entry) => `guide/ui/${entry.canonicalName}.md`)
    .filter((page) => !fs.existsSync(path.join(docsContentRoot, page)));
  assert.deepEqual(missing, [], "run: node npm/ui/scripts/generate-reference-docs.ts");
});

void test("every composable catalog entry has a reference page", () => {
  const pages = referencedComposables().map(
    (entry) => `guide/composables/${entry.subpath.slice(2)}.md`,
  );
  assert.ok(pages.length > 0);
  assert.equal(pages.length, COMPOSABLE_CATALOG.entries.length - 1, "only ./catalog is excluded");
  const missing = pages.filter((page) => !fs.existsSync(path.join(docsContentRoot, page)));
  assert.deepEqual(missing, [], "run: node npm/ui/scripts/generate-reference-docs.ts");
});

void test("committed reference pages match the generator and have no orphans", () => {
  const stale = [...rendered]
    .filter(([page, content]) => {
      const target = path.join(docsContentRoot, page);
      return !fs.existsSync(target) || fs.readFileSync(target, "utf8") !== content;
    })
    .map(([page]) => page);
  assert.deepEqual(stale, [], "run: node npm/ui/scripts/generate-reference-docs.ts");
  const orphans = existingReferenceDocs().filter((page) => !rendered.has(page));
  assert.deepEqual(orphans, [], "pages without a catalog entry must be removed");
});

void test("index pages link every reference page and are in the sidebar", () => {
  const uiIndex = rendered.get("guide/ui/index.md") ?? "";
  for (const entry of uiFamilyCatalog) {
    assert.ok(uiIndex.includes(`(./${entry.canonicalName}.md)`), entry.canonicalName);
  }
  const composableIndex = rendered.get("guide/composables/index.md") ?? "";
  for (const entry of referencedComposables()) {
    assert.ok(composableIndex.includes(`(./${entry.subpath.slice(2)}.md)`), entry.subpath);
  }
  const navPaths = sitemap.navGroups.flatMap((group) => group.paths);
  assert.ok(navPaths.includes("/guide/ui"));
  assert.ok(navPaths.includes("/guide/composables"));
});

void test("component pages document the SFC contract extracted from source", () => {
  const page = rendered.get("guide/ui/switch.md") ?? "";
  assert.match(page, /#### Props\n\n\| Prop \| Type \| Default \| Description \|/);
  assert.match(page, /\| `modelValue` \| `boolean` \| `undefined` \| Controlled checked value/);
  assert.match(page, /\| `update:modelValue` \| `\[value: boolean\]` \|/);
  assert.match(page, /\| `default` \| `SwitchSlotState` \|/);
  assert.match(page, /\| `toggle` \| `\(\) => boolean` \|/);
  assert.match(page, /## Behavior/);
  const composable = rendered.get("guide/composables/use-toggle.md") ?? "";
  assert.match(composable, /function useToggle\(/);
  assert.match(composable, /\| `useToggle` \| state \|/);
});
