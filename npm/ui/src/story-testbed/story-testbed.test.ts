import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import { runUiStoryTestbedCli } from "../../scripts/story-testbed.ts";
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
  type UiStoryTestbedEntry,
  type UiStoryTestbedFamilySummary,
  type UiStoryTestbedPlanItem,
} from "./story-testbed.ts";
import { themePresets } from "../families/foundations/theme/theme-constants.ts";

interface ListJsonOutput {
  readonly schemaVersion: number;
  readonly command: "list";
  readonly familyCount: number;
  readonly families: readonly UiStoryTestbedFamilySummary[];
}

interface PlanJsonOutput {
  readonly schemaVersion: number;
  readonly command: "plan";
  readonly surface: string | null;
  readonly planCount: number;
  readonly plan: readonly UiStoryTestbedPlanItem[];
}

interface InfoJsonOutput {
  readonly schemaVersion: number;
  readonly command: "info";
  readonly family: UiStoryTestbedEntry;
}

interface CheckJsonOutput {
  readonly schemaVersion: number;
  readonly command: "check";
  readonly familyCount: number;
  readonly planCount: number;
  readonly surfaces: readonly string[];
  readonly browserSuites: readonly string[];
  readonly harnessHooks: readonly string[];
  readonly violationCount: number;
}

interface CapturedCli {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
}

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

function runCli(args: readonly string[]): CapturedCli {
  let stdout = "";
  let stderr = "";
  const exitCode = runUiStoryTestbedCli(args, {
    stdout: {
      write(chunk) {
        stdout += chunk;
      },
    },
    stderr: {
      write(chunk) {
        stderr += chunk;
      },
    },
  });

  return { exitCode, stdout, stderr };
}

function parseJson<Output>(source: string): Output {
  return JSON.parse(source) as Output;
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
  const existingFiles = await collectSourceFiles(path.resolve("src"));
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

test("CLI emits deterministic machine-readable story-testbed output", () => {
  const helpOutput = runCli(["--help"]);
  assert.equal(helpOutput.exitCode, 0);
  assert.match(helpOutput.stdout, /repository checkout only/);
  assert.match(helpOutput.stdout, /not a published @vizejs\/ui package bin/);
  assert.match(helpOutput.stdout, /issue #4898/);

  const checkOutput = runCli(["check", "--format", "json"]);
  assert.equal(checkOutput.exitCode, 0);
  assert.equal(checkOutput.stderr, "");
  assert.ok(checkOutput.stdout.length < 2_000, "check output must stay compact for CI logs");
  const checked = parseJson<CheckJsonOutput>(checkOutput.stdout);
  assert.equal(checked.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(checked.command, "check");
  assert.equal(checked.familyCount, uiFamilyCatalog.length);
  assert.equal(checked.planCount, uiFamilyCatalog.length * uiStoryTestbedSurfaces.length);
  assert.deepEqual(checked.surfaces, uiStoryTestbedSurfaces);
  assert.deepEqual(checked.browserSuites, uiStoryTestbedBrowserSuiteNames);
  assert.deepEqual(checked.harnessHooks, uiStoryTestbedHarnessHookNames);
  assert.equal(checked.violationCount, 0);

  const listOutput = runCli(["list", "--format", "json"]);
  assert.equal(listOutput.exitCode, 0);
  assert.equal(listOutput.stderr, "");
  const listed = parseJson<ListJsonOutput>(listOutput.stdout);
  assert.equal(listed.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(listed.command, "list");
  assert.equal(listed.familyCount, uiFamilyCatalog.length);
  assert.deepEqual(
    listed.families.map((family) => family.canonicalName),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );

  const planOutput = runCli(["plan", "vitest-browser"]);
  assert.equal(planOutput.exitCode, 0);
  assert.equal(planOutput.stderr, "");
  const plan = parseJson<PlanJsonOutput>(planOutput.stdout);
  assert.equal(plan.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(plan.command, "plan");
  assert.equal(plan.surface, "vitest-browser");
  assert.equal(plan.planCount, uiFamilyCatalog.length);
  assert.ok(plan.plan.every((item) => item.runner === "vitest-browser"));

  const shorthandPlanOutput = runCli(["plan", "--surface=playwright-vrt", "--jsonl"]);
  assert.equal(shorthandPlanOutput.exitCode, 0);
  assert.equal(shorthandPlanOutput.stderr, "");
  const planLines = shorthandPlanOutput.stdout.trim().split("\n");
  assert.equal(planLines.length, uiFamilyCatalog.length);
  assert.ok(
    planLines.every(
      (line) => parseJson<{ readonly surface: string }>(line).surface === "playwright-vrt",
    ),
  );

  const infoOutput = runCli(["info", "./button"]);
  assert.equal(infoOutput.exitCode, 0);
  assert.equal(infoOutput.stderr, "");
  const info = parseJson<InfoJsonOutput>(infoOutput.stdout);
  assert.equal(info.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(info.command, "info");
  assert.equal(info.family.canonicalName, "button");
  assert.equal(info.family.storyFile, "src/families/actions/button/button.art.vue");
});

test("behavior contract documents the issue 4898 harness gates", async () => {
  const behavior = await readFile(
    path.resolve("src/story-testbed/story-testbed.behavior.md"),
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
