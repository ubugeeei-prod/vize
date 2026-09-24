import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import {
  referencedComposables,
  renderReferenceDocs,
} from "../../npm/ui/scripts/generate-reference-docs.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";
import { repoRoot } from "./_helpers/moonbit.ts";

await import("../../docs/theme/i18n/sitemap.js");
const sitemap = (
  globalThis as { __vizeDocsSitemap?: { navGroups: Array<{ key: string; paths: string[] }> } }
).__vizeDocsSitemap!;

// Reference pages are rendered at docs build time (`pnpm generate:reference`
// in docs/), never committed, so catalog changes cannot leave stale pages or
// conflict on the shared index pages.
const rendered = renderReferenceDocs();

void test("every UI catalog family renders a reference page", () => {
  const missing = uiFamilyCatalog
    .map((entry) => `guide/ui/${entry.canonicalName}.md`)
    .filter((page) => !rendered.has(page));
  assert.deepEqual(missing, []);
});

void test("every composable catalog entry renders a reference page", () => {
  const pages = referencedComposables().map(
    (entry) => `guide/composables/${entry.subpath.slice(2)}.md`,
  );
  assert.equal(pages.length, COMPOSABLE_CATALOG.entries.length - 1, "only ./catalog is excluded");
  assert.deepEqual(
    pages.filter((page) => !rendered.has(page)),
    [],
  );
});

void test("generated reference pages are build outputs, not committed files", () => {
  const tracked = spawnSync(
    "git",
    [
      "ls-files",
      "--",
      "docs/content/guide/ui",
      "docs/content/guide/composables",
      "docs/content/*/guide/ui",
      "docs/content/*/guide/composables",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(tracked.status, 0, tracked.stderr);
  assert.equal(
    tracked.stdout.trim(),
    "",
    "remove committed reference pages; docs build generates them",
  );
});

void test("index pages link every reference page in every locale and are in the sidebar", () => {
  for (const prefix of ["", "ja/", "zh-CN/", "pt-BR/", "fr/"]) {
    const uiIndex = rendered.get(`${prefix}guide/ui/index.md`) ?? "";
    const composableIndex = rendered.get(`${prefix}guide/composables/index.md`) ?? "";
    for (const entry of uiFamilyCatalog) {
      assert.ok(uiIndex.includes(`${entry.canonicalName}.md)`), `${prefix}${entry.canonicalName}`);
    }
    for (const entry of referencedComposables()) {
      assert.ok(
        composableIndex.includes(`${entry.subpath.slice(2)}.md)`),
        `${prefix}${entry.subpath}`,
      );
    }
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
