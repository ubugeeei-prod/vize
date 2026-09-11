import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "../catalog/family-catalog.ts";
import {
  UI_STORY_TESTBED_PACKAGE_NAME,
  auditUiStoryTestbedInventory,
  createUiStoryTestbedManifest,
  formatUiStoryTestbedViolations,
  getUiStoryTestbedFamilyInfo,
  listUiStoryTestbedFamilies,
  listUiStoryTestbedPlan,
  UI_STORY_TESTBED_SCHEMA_VERSION,
  uiStoryTestbedBrowserSuiteNames,
  uiStoryTestbedHarnessHookNames,
  uiStoryMatrixDimensions,
  uiStoryTestbedInventory,
  uiStoryTestbedScreenshotReview,
  uiStoryTestbedSurfaces,
  uiStoryTestbedViewports,
} from "./story-testbed.ts";
import { themePresets } from "../families/foundations/theme/theme-constants.ts";

const uiRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function colocatedArtifactFile(
  targetFile: `src/${string}`,
  canonicalName: string,
  suffix: string,
): string {
  return path.posix.join(path.posix.dirname(targetFile), `${canonicalName}${suffix}`);
}

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

function jsonRoundTrip<Value>(value: Value): Value {
  return JSON.parse(JSON.stringify(value)) as Value;
}

test("publishes a deterministic story-testbed inventory for each family", () => {
  assert.equal(UI_STORY_TESTBED_SCHEMA_VERSION, 2);
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
    const primaryTarget = entry.targetFiles[0] ?? catalogEntry.entryFile;
    assert.equal(
      entry.storyFile,
      colocatedArtifactFile(primaryTarget, entry.canonicalName, ".art.vue"),
    );
    assert.equal(
      entry.vueTestFile,
      colocatedArtifactFile(primaryTarget, entry.canonicalName, ".vue.test.ts"),
    );
    assert.equal(
      entry.browserTestFile,
      colocatedArtifactFile(primaryTarget, entry.canonicalName, ".browser.spec.ts"),
    );
    assert.equal(
      entry.vrtTestFile,
      colocatedArtifactFile(primaryTarget, entry.canonicalName, ".vrt.spec.ts"),
    );
    assert.deepEqual(entry.supportingTestFiles, catalogEntry.tests);
    assert.deepEqual(entry.matrixDimensions, uiStoryMatrixDimensions);
    assert.deepEqual(entry.presets, themePresets);
    assert.deepEqual(entry.viewports, uiStoryTestbedViewports);
    assert.deepEqual(
      entry.browserSuites.map((suite) => suite.name),
      uiStoryTestbedBrowserSuiteNames,
    );
    assert.deepEqual(
      entry.harnessHooks.map((hook) => hook.name),
      uiStoryTestbedHarnessHookNames,
    );
    assert.deepEqual(entry.screenshotReview, uiStoryTestbedScreenshotReview);

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

test("publishes a machine-readable harness manifest and run plan", () => {
  const manifest = createUiStoryTestbedManifest();
  const manifestAgain = createUiStoryTestbedManifest();

  assert.equal(manifest.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(manifest.packageName, UI_STORY_TESTBED_PACKAGE_NAME);
  assert.equal(manifest.sourceRoot, "npm/ui");
  assert.equal(JSON.stringify(manifest), JSON.stringify(manifestAgain));
  assert.deepEqual(manifest.surfaces, uiStoryTestbedSurfaces);
  assert.deepEqual(manifest.matrixDimensions, uiStoryMatrixDimensions);
  assert.deepEqual(manifest.presets, themePresets);
  assert.deepEqual(manifest.viewports, uiStoryTestbedViewports);
  assert.deepEqual(
    manifest.browserSuites.map((suite) => suite.name),
    ["focus", "pointer", "layout"],
  );
  assert.deepEqual(
    manifest.harnessHooks.map((hook) => hook.name),
    [
      "accessibility-tree",
      "live-region-transcript",
      "event-log",
      "bundle-explorer",
      "ssr-hydration-lab",
    ],
  );
  assert.equal(manifest.screenshotReview.artifactDirectory, ".vize/artifacts/ui-vrt");

  const summaries = listUiStoryTestbedFamilies(manifest);
  assert.deepEqual(
    summaries.map((summary) => summary.canonicalName),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );
  assert.equal(getUiStoryTestbedFamilyInfo("./button", manifest)?.canonicalName, "button");

  const fullPlan = listUiStoryTestbedPlan({}, manifest);
  assert.equal(fullPlan.length, uiFamilyCatalog.length * uiStoryTestbedSurfaces.length);

  const browserPlan = listUiStoryTestbedPlan({ surface: "vitest-browser" }, manifest);
  assert.equal(browserPlan.length, uiFamilyCatalog.length);
  assert.ok(browserPlan.every((item) => item.surface === "vitest-browser"));
  assert.ok(browserPlan.every((item) => item.runner === "vitest-browser"));
  assert.ok(
    browserPlan.every(
      (item) => item.browserSuites.map((suite) => suite.name).join(",") === "focus,pointer,layout",
    ),
  );

  const vrtPlan = listUiStoryTestbedPlan({ surface: "playwright-vrt" }, manifest);
  assert.ok(vrtPlan.every((item) => item.screenshotReview.workflow === "review-artifacts"));
});

test("audits planned artifacts and supporting tests against source files", async () => {
  const existingFiles = await collectSourceFiles(path.join(uiRoot, "src"));
  const violations = auditUiStoryTestbedInventory(uiStoryTestbedInventory, { existingFiles });

  assert.equal(formatUiStoryTestbedViolations(violations), "");
  assert.deepEqual(violations, []);

  const plannedArtifacts = uiStoryTestbedInventory.flatMap((entry) =>
    entry.artifacts.filter((artifact) => artifact.status === "planned"),
  );
  const supportingTestFiles = uiStoryTestbedInventory.flatMap((entry) => entry.supportingTestFiles);
  const hookNames = uiStoryTestbedInventory.flatMap((entry) =>
    entry.harnessHooks.map((hook) => hook.name),
  );
  assert.equal(plannedArtifacts.length, uiFamilyCatalog.length * 4);
  assert.ok(supportingTestFiles.length >= uiFamilyCatalog.length);
  assert.equal(hookNames.length, uiFamilyCatalog.length * uiStoryTestbedHarnessHookNames.length);
});

test("audits shared story-testbed contracts structurally after JSON round trips", () => {
  const roundTrippedInventory = jsonRoundTrip(uiStoryTestbedInventory);
  const violations = auditUiStoryTestbedInventory(roundTrippedInventory);

  assert.deepEqual(violations, []);
});

test("audits ready artifacts for listed evidence and canonical colocated files", () => {
  const [entry] = uiStoryTestbedInventory;
  assert.ok(entry, "story-testbed inventory must include at least one family");

  const emptyReady = {
    ...entry,
    artifacts: entry.artifacts.map((artifact) =>
      artifact.surface === "musea-story"
        ? { ...artifact, status: "ready" as const, files: [] }
        : artifact,
    ),
  };
  const wrongReady = {
    ...entry,
    artifacts: entry.artifacts.map((artifact) =>
      artifact.surface === "musea-story"
        ? { ...artifact, status: "ready" as const, files: [entry.vueTestFile] }
        : artifact,
    ),
  };

  assert.ok(
    auditUiStoryTestbedInventory([emptyReady]).some(
      (violation) =>
        violation.code === "ready-artifact-missing" &&
        violation.message.includes("does not list evidence files"),
    ),
  );
  assert.ok(
    auditUiStoryTestbedInventory([wrongReady]).some(
      (violation) =>
        violation.code === "ready-artifact-missing" && violation.message.includes(entry.storyFile),
    ),
  );
});

test("behavior contract documents the issue 4898 harness gates", async () => {
  const behavior = await readFile(
    path.join(uiRoot, "src/story-testbed/story-testbed.behavior.md"),
    "utf8",
  );

  assert.match(behavior, /S1.+family catalog.+stable public family/);
  assert.match(behavior, /S2.+Musea surface.+<target-dir>\/<family>\.art\.vue/);
  assert.match(
    behavior,
    /S3.+states, slots, parts, presets, RTL, reduced-motion, and forced-colors/,
  );
  assert.match(behavior, /S4.+supporting behavior tests must exist/);
  assert.match(behavior, /S5.+focus, pointer, and layout/);
  assert.match(behavior, /S6.+accessibility tree, live-region transcript, event log/);
  assert.match(behavior, /S7.+screenshot review/);
});
