import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import {
  referencedComposables,
  renderReferenceDocs,
} from "../../npm/ui/scripts/generate-reference-docs.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";
import { featuredExamples, publicExample } from "../../npm/ui/scripts/reference-docs/examples.ts";
import {
  composableExamples,
  composableExampleSource,
} from "../../npm/ui/scripts/reference-docs/composable-examples.ts";
import { previewComposableExamples } from "../../docs/previews/ui/build-config.ts";
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

void test("type-only composable pages show usable imports and type contracts", () => {
  const scheduler = rendered.get("guide/composables/timeout-scheduler.md") ?? "";
  assert.match(scheduler, /import type \{ TimeoutScheduler \} from/);
  assert.match(scheduler, /Single-shot timer host/);
  assert.doesNotMatch(scheduler, /Provides \.|import \{  \}/);

  const watchSource = rendered.get("guide/composables/watch-source.md") ?? "";
  assert.match(watchSource, /import type \{ WatchSourceInput, WatchSources,/);
  assert.match(watchSource, /### `WatchHelperCallback`/);
  assert.doesNotMatch(watchSource, /Provides \.|import \{  \}/);
});

void test("all maintained basic examples preserve their template and use published import paths", () => {
  const uiRoot = path.join(repoRoot, "npm/ui");
  const exports = JSON.parse(readFileSync(path.join(uiRoot, "package.json"), "utf8")).exports;
  let examples = 0;
  for (const entry of uiFamilyCatalog) {
    const source = publicExample(uiRoot, entry);
    if (source == null) continue;
    examples += 1;
    const original = readFileSync(
      path.join(
        uiRoot,
        path.dirname(entry.entryFile),
        "examples",
        `${entry.canonicalName}-basic.vue`,
      ),
      "utf8",
    );
    assert.equal(
      source.slice(source.indexOf("<template>")),
      original.slice(original.indexOf("<template>")),
      entry.canonicalName,
    );
    assert.doesNotMatch(
      source,
      /from ["']\./,
      `${entry.canonicalName}: no repository-relative imports`,
    );
    for (const match of source.matchAll(/from ["']@vizejs\/ui([^"']*)["']/g)) {
      assert.ok(
        exports[match[1] === "" ? "." : `.${match[1]}`],
        `${entry.canonicalName}: ${match[0]}`,
      );
    }
    const page = rendered.get(`guide/ui/${entry.canonicalName}.md`) ?? "";
    assert.ok(page.includes(source.trim()), `${entry.canonicalName}: complete copyable SFC`);
    assert.ok(
      page.includes(`index.html?family=${entry.canonicalName}`),
      `${entry.canonicalName}: same live example`,
    );
  }
  assert.ok(examples >= 140, `maintained component examples: ${examples}`);
});

void test("component hub guides tasks in English and Japanese and links real captures", () => {
  const en = rendered.get("guide/ui/index.md") ?? "";
  const ja = rendered.get("ja/guide/ui/index.md") ?? "";
  assert.match(en, /Collect user input/);
  assert.match(en, /Ask for confirmation or add context/);
  assert.match(ja, /入力を受け取る/);
  assert.match(ja, /確認・補足を表示する/);
  assert.match(en, /vp install @vizejs\/ui/);
  for (const family of featuredExamples) {
    for (const hub of [en, ja])
      assert.ok(hub.includes(`/component-previews/${family}.png`), family);
    assert.ok(
      (rendered.get(`guide/ui/${family}.md`) ?? "").includes(`/component-previews/${family}.png`),
      family,
    );
  }
});

void test("practical composable examples share complete source with their live preview", () => {
  const packageRoot = path.join(repoRoot, "npm/compose/core");
  const exports = JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8")).exports;
  assert.equal(composableExamples.length, 12);
  assert.equal(new Set(composableExamples.map((example) => example.name)).size, 12);
  for (const example of composableExamples) {
    const source = composableExampleSource(packageRoot, example.name);
    const preview = previewComposableExamples.find((item) => item.name === example.name);
    assert.equal(preview?.source, source, `${example.name}: exact runtime source`);
    const page = rendered.get(`guide/composables/${example.name}.md`) ?? "";
    assert.ok(page.includes(source.trim()), `${example.name}: complete displayed SFC`);
    assert.ok(
      page.includes(`index.html?composable=${example.name}`),
      `${example.name}: live preview`,
    );
    for (const explanation of [example.purpose, example.observe, example.context])
      assert.ok(page.includes(explanation));
    assert.doesNotMatch(source, /from ["']\./, "examples must be usable in a consumer project");
    for (const match of source.matchAll(/from ["']@vizejs\/composable\/([^"']+)["']/g))
      assert.ok(exports[`./${match[1]}`], `${example.name}: public import`);
  }
  assert.throws(
    () => composableExampleSource(packageRoot, "missing-public-entry"),
    /no public composable entry/,
  );
});

void test("all reference entries show setup and composable hubs disclose preview coverage", () => {
  for (const [file, page] of rendered) {
    if (/^guide\/(ui|composables)\/(?!index\.md)/.test(file))
      assert.match(page, /## Minimal setup/, file);
  }
  for (const prefix of ["", "ja/", "zh-CN/", "pt-BR/", "fr/"]) {
    const hub = rendered.get(`${prefix}guide/composables/index.md`) ?? "";
    assert.match(hub, /vp install @vizejs\/composable/);
    for (const example of composableExamples)
      assert.ok(hub.includes(`${example.name}.md)`), `${prefix}${example.name}`);
  }
  assert.match(
    rendered.get("guide/composables/index.md") ?? "",
    new RegExp(`checks currently cover the ${composableExamples.length} examples`),
  );
});
