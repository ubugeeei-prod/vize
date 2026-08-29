import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "./family-catalog.ts";
import {
  auditUiStoryTestbedInventory,
  formatUiStoryTestbedViolations,
  UI_STORY_TESTBED_SCHEMA_VERSION,
  uiStoryMatrixDimensions,
  uiStoryTestbedInventory,
  uiStoryTestbedSurfaces,
  uiStoryTestbedViewports,
} from "./story-testbed.ts";
import { themePresets } from "./theme-constants.ts";

async function collectSourceFiles(
  directory: string,
  relativeDirectory = "src",
): Promise<ReadonlySet<string>> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry): Promise<readonly string[]> => {
      const entryPath = path.join(directory, entry.name);
      const relativePath = path.posix.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) return [...(await collectSourceFiles(entryPath, relativePath))];
      return entry.isFile() ? [relativePath] : [];
    }),
  );

  return new Set(files.flat().sort((left, right) => left.localeCompare(right)));
}

test("publishes a deterministic story-testbed inventory for each family", () => {
  assert.equal(UI_STORY_TESTBED_SCHEMA_VERSION, 1);
  assert.equal(uiStoryTestbedInventory.length, uiFamilyCatalog.length);

  const catalogNames = uiFamilyCatalog.map((entry) => entry.canonicalName);
  const inventoryNames = uiStoryTestbedInventory.map((entry) => entry.canonicalName);
  assert.deepEqual(inventoryNames, catalogNames);

  const catalogByName = new Map(uiFamilyCatalog.map((entry) => [entry.canonicalName, entry]));
  for (const entry of uiStoryTestbedInventory) {
    const catalogEntry = catalogByName.get(entry.canonicalName);
    assert.ok(catalogEntry, `${entry.canonicalName} must exist in the family catalog`);

    assert.equal(entry.title, catalogEntry.title);
    assert.equal(entry.packageSubpath, catalogEntry.packageSubpath);
    assert.equal(entry.storyFile, `src/${entry.canonicalName}.art.vue`);
    assert.equal(entry.vueTestFile, `src/${entry.canonicalName}.vue.test.ts`);
    assert.equal(entry.browserTestFile, `src/${entry.canonicalName}.browser.spec.ts`);
    assert.equal(entry.vrtTestFile, `src/${entry.canonicalName}.vrt.spec.ts`);
    assert.deepEqual(entry.supportingTestFiles, catalogEntry.tests);
    assert.deepEqual(entry.matrixDimensions, uiStoryMatrixDimensions);
    assert.deepEqual(entry.presets, themePresets);
    assert.deepEqual(entry.viewports, uiStoryTestbedViewports);

    for (const targetFile of entry.targetFiles) {
      assert.ok(
        catalogEntry.sourceFiles.includes(targetFile) || targetFile === catalogEntry.entryFile,
        `${entry.canonicalName} target ${targetFile} must come from the family catalog`,
      );
      assert.equal(
        path.posix.dirname(targetFile),
        path.posix.dirname(entry.storyFile),
        `${entry.canonicalName} story must stay colocated with its target files`,
      );
    }

    const artifactsBySurface = new Map(
      entry.artifacts.map((artifact) => [artifact.surface, artifact]),
    );
    assert.deepEqual([...artifactsBySurface.keys()], uiStoryTestbedSurfaces);
    assert.equal(artifactsBySurface.get("musea-story")?.status, "planned");
    assert.equal(artifactsBySurface.get("vue-test-utils")?.status, "planned");
    assert.equal(artifactsBySurface.get("vitest-browser")?.status, "planned");
    assert.equal(artifactsBySurface.get("playwright-vrt")?.status, "planned");
  }
});

test("audits planned artifacts and supporting tests against source files", async () => {
  const existingFiles = await collectSourceFiles(path.resolve("src"));
  const violations = auditUiStoryTestbedInventory(uiStoryTestbedInventory, { existingFiles });

  assert.equal(formatUiStoryTestbedViolations(violations), "");
  assert.deepEqual(violations, []);

  const plannedArtifacts = uiStoryTestbedInventory.flatMap((entry) =>
    entry.artifacts.filter((artifact) => artifact.status === "planned"),
  );
  const supportingTestFiles = uiStoryTestbedInventory.flatMap((entry) => entry.supportingTestFiles);
  assert.equal(plannedArtifacts.length, uiFamilyCatalog.length * 4);
  assert.ok(supportingTestFiles.length >= uiFamilyCatalog.length);
});

test("behavior contract documents the issue 4898 harness gates", async () => {
  const behavior = await readFile(path.resolve("src/story-testbed.behavior.md"), "utf8");

  assert.match(behavior, /S1.+family catalog.+stable public family/);
  assert.match(behavior, /S2.+Musea surface.+src\/<family>\.art\.vue/);
  assert.match(
    behavior,
    /S3.+states, slots, parts, presets, RTL, reduced-motion, and forced-colors/,
  );
  assert.match(behavior, /S4.+supporting behavior tests must exist/);
});
