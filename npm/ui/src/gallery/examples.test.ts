import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";
import { createApp, createSSRApp, type Component } from "vue";
import { renderToString } from "vue/server-renderer";

import { familyExamples, renderGallery } from "../../scripts/generate-gallery.ts";
import { uiLibRegistryOptions } from "../../scripts/build-source-registry.ts";
import { componentExports } from "../../scripts/reference-docs/extract.ts";
import { buildLibRegistry } from "../../scripts/source-registry-bundle/bundle.ts";
import { exampleDescription } from "../../scripts/source-registry-bundle/examples.ts";
import { createPackageLibRegistryInput } from "../../scripts/source-registry-bundle/write.ts";
import { uiFamilyCatalog } from "../catalog/family-catalog.ts";

const packageRoot = path.resolve(import.meta.dirname, "../..");
const modules = import.meta.glob<{ default: Component }>("../families/*/*/examples/*.vue", {
  eager: true,
});
const examples = Object.entries(modules).map(([file, module]) => ({
  file: file.replace(/^\.\.\//, "src/"),
  component: module.default,
}));

function familiesWithComponents() {
  return uiFamilyCatalog.filter(
    (entry) => componentExports(path.join(packageRoot, entry.entryFile)).length > 0,
  );
}

function captureWarnings(run: () => Promise<void>): Promise<readonly string[]> {
  const warnings: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...args: unknown[]) => warnings.push(args.map(String).join(" "));
  console.error = (...args: unknown[]) => warnings.push(args.map(String).join(" "));
  return run()
    .then(() => warnings)
    .finally(() => {
      console.warn = originalWarn;
      console.error = originalError;
    });
}

/**
 * Every family that exports components must ship
 * `examples/<family>-basic.vue` (also enforced by `pnpm check:examples`).
 */
const REQUIRE_EXAMPLE_FOR_EVERY_FAMILY = true;

test("every family that exports components ships at least one example", () => {
  const missing = familiesWithComponents()
    .filter((entry) => familyExamples(entry).length === 0)
    .map((entry) => entry.canonicalName);
  if (REQUIRE_EXAMPLE_FOR_EVERY_FAMILY) {
    assert.deepEqual(missing, [], "add src/families/<area>/<family>/examples/<family>-basic.vue");
  }
  const covered = familiesWithComponents().length - missing.length;
  assert.ok(covered >= 140, `example coverage regressed to ${covered} families`);
});

test("examples start with a description comment and import only their closure", () => {
  const bundle = buildLibRegistry(createPackageLibRegistryInput(uiLibRegistryOptions()));
  const published = bundle.manifest.items.flatMap((item) =>
    item.examples.map((example) => example.path),
  );
  for (const entry of uiFamilyCatalog) {
    for (const example of familyExamples(entry)) {
      const relative = path
        .relative(path.join(packageRoot, "src"), example)
        .split(path.sep)
        .join("/");
      assert.notEqual(exampleDescription(readFileSync(example, "utf8")), "", relative);
      assert.ok(published.includes(relative), `${relative} ships in the registry`);
    }
  }
  assert.ok(examples.length > 0);
});

test("examples render identically across SSR requests and hydrate without warnings", async () => {
  for (const { file, component } of examples) {
    const warnings = await captureWarnings(async () => {
      const [first, second] = await Promise.all([
        renderToString(createSSRApp(component)),
        renderToString(createSSRApp(component)),
      ]);
      assert.equal(first, second, `${file} renders deterministically`);
      assert.ok((first ?? "").length > 0, `${file} renders markup`);
      const host = document.createElement("div");
      host.innerHTML = first ?? "";
      document.body.append(host);
      const app = createSSRApp(component);
      app.mount(host);
      app.unmount();
      host.remove();
    });
    assert.deepEqual(warnings, [], `${file} warns during SSR or hydration`);
  }
});

test("examples mount client-side without warnings", async () => {
  for (const { file, component } of examples) {
    const warnings = await captureWarnings(async () => {
      const host = document.createElement("div");
      document.body.append(host);
      const app = createApp(component);
      app.mount(host);
      assert.ok(host.innerHTML.length > 0, `${file} renders markup`);
      app.unmount();
      host.remove();
    });
    assert.deepEqual(warnings, [], `${file} warns when mounted`);
  }
});

test("the Musea gallery gets one story per family with one variant per example", () => {
  const stories = renderGallery();
  const families = new Set(stories.map((story) => story.family));
  for (const entry of familiesWithComponents()) {
    assert.equal(
      families.has(entry.canonicalName),
      familyExamples(entry).length > 0,
      `${entry.canonicalName} has a story exactly when it ships examples`,
    );
  }
  const switchStory = stories.find((story) => story.family === "switch");
  assert.ok(switchStory);
  assert.equal(switchStory.file, "selection/switch.art.vue");
  assert.match(
    switchStory.source,
    /defineArt\("\.\.\/\.\.\/\.\.\/src\/families\/selection\/switch\/switch-control\.vue"/,
  );
  assert.match(switchStory.source, /<variant name="Basic" default>\n {4}<SwitchBasicExample \/>/);
  const variants = stories.reduce((count, story) => count + story.examples.length, 0);
  assert.equal(variants, examples.length);
});
