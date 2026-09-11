import { uiFamilyCatalog, type UiFamilyCatalogEntry } from "../catalog/family-catalog.ts";
import { themePresets } from "../families/foundations/theme/theme-constants.ts";
import {
  browserTestFileFor,
  storyFileFor,
  vrtTestFileFor,
  vueTestFileFor,
} from "./story-testbed-paths.ts";
import {
  UI_STORY_TESTBED_PACKAGE_NAME,
  UI_STORY_TESTBED_SCHEMA_VERSION,
  UI_STORY_TESTBED_SOURCE_ROOT,
  uiStoryMatrixDimensions,
  uiStoryTestbedBrowserSuites,
  uiStoryTestbedHarnessHooks,
  uiStoryTestbedScreenshotReview,
  uiStoryTestbedSurfaces,
  uiStoryTestbedViewports,
  type UiStoryTestbedEntry,
  type UiStoryTestbedFamilySummary,
  type UiStoryTestbedManifest,
  type UiStoryTestbedPlanItem,
  type UiStoryTestbedPlanOptions,
  type UiStoryTestbedRunner,
  type UiStoryTestbedSurface,
} from "./story-testbed-contract.ts";

function isVueSourceFile(file: `src/${string}`): file is `src/${string}.vue` {
  return file.endsWith(".vue");
}

function testbedTargetsFor(entry: UiFamilyCatalogEntry): readonly `src/${string}`[] {
  const componentTargets = entry.sourceFiles.filter(isVueSourceFile);
  return componentTargets.length > 0 ? componentTargets : [entry.entryFile];
}

function createStoryTestbedEntry(entry: UiFamilyCatalogEntry): UiStoryTestbedEntry {
  const targetFiles = testbedTargetsFor(entry);
  const primaryTarget = targetFiles[0] ?? entry.entryFile;
  const storyFile = storyFileFor(entry.canonicalName, primaryTarget);
  const vueTestFile = vueTestFileFor(entry.canonicalName, primaryTarget);
  const browserTestFile = browserTestFileFor(entry.canonicalName, primaryTarget);
  const vrtTestFile = vrtTestFileFor(entry.canonicalName, primaryTarget);

  return {
    canonicalName: entry.canonicalName,
    title: entry.title,
    packageSubpath: entry.packageSubpath,
    targetFiles,
    storyFile,
    vueTestFile,
    browserTestFile,
    vrtTestFile,
    supportingTestFiles: entry.tests,
    matrixDimensions: uiStoryMatrixDimensions,
    presets: themePresets,
    viewports: uiStoryTestbedViewports,
    browserSuites: uiStoryTestbedBrowserSuites,
    harnessHooks: uiStoryTestbedHarnessHooks,
    screenshotReview: uiStoryTestbedScreenshotReview,
    artifacts: [
      { surface: "musea-story", status: "planned", files: [storyFile] },
      { surface: "vue-test-utils", status: "planned", files: [vueTestFile] },
      { surface: "vitest-browser", status: "planned", files: [browserTestFile] },
      { surface: "playwright-vrt", status: "planned", files: [vrtTestFile] },
    ],
  };
}

export const uiStoryTestbedInventory: readonly UiStoryTestbedEntry[] =
  uiFamilyCatalog.map(createStoryTestbedEntry);

export function createUiStoryTestbedManifest(
  inventory: readonly UiStoryTestbedEntry[] = uiStoryTestbedInventory,
): UiStoryTestbedManifest {
  return {
    schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
    packageName: UI_STORY_TESTBED_PACKAGE_NAME,
    sourceRoot: UI_STORY_TESTBED_SOURCE_ROOT,
    surfaces: uiStoryTestbedSurfaces,
    matrixDimensions: uiStoryMatrixDimensions,
    presets: themePresets,
    viewports: uiStoryTestbedViewports,
    browserSuites: uiStoryTestbedBrowserSuites,
    harnessHooks: uiStoryTestbedHarnessHooks,
    screenshotReview: uiStoryTestbedScreenshotReview,
    families: inventory,
  };
}

export function summarizeUiStoryTestbedFamily(
  entry: UiStoryTestbedEntry,
): UiStoryTestbedFamilySummary {
  const readyArtifactCount = entry.artifacts.filter(
    (artifact) => artifact.status === "ready",
  ).length;
  const plannedArtifactCount = entry.artifacts.length - readyArtifactCount;

  return {
    canonicalName: entry.canonicalName,
    title: entry.title,
    packageSubpath: entry.packageSubpath,
    targetFileCount: entry.targetFiles.length,
    plannedArtifactCount,
    readyArtifactCount,
    surfaces: entry.artifacts.map((artifact) => artifact.surface),
  };
}

export function listUiStoryTestbedFamilies(
  manifest: UiStoryTestbedManifest = createUiStoryTestbedManifest(),
): readonly UiStoryTestbedFamilySummary[] {
  return manifest.families.map(summarizeUiStoryTestbedFamily);
}

export function getUiStoryTestbedFamilyInfo(
  nameOrSubpath: string,
  manifest: UiStoryTestbedManifest = createUiStoryTestbedManifest(),
): UiStoryTestbedEntry | undefined {
  const raw = nameOrSubpath.trim();
  if (raw.length === 0) return undefined;

  const normalized = raw.toLowerCase();
  return manifest.families.find(
    (entry) =>
      entry.canonicalName === raw ||
      entry.packageSubpath === raw ||
      entry.packageSubpath.slice(2) === raw ||
      entry.title.toLowerCase() === normalized,
  );
}

function runnerForSurface(surface: UiStoryTestbedSurface): UiStoryTestbedRunner {
  switch (surface) {
    case "musea-story":
      return "musea";
    case "vue-test-utils":
      return "vue-test-utils";
    case "vitest-browser":
      return "vitest-browser";
    case "playwright-vrt":
      return "playwright";
  }
}

export function listUiStoryTestbedPlan(
  options: UiStoryTestbedPlanOptions = {},
  manifest: UiStoryTestbedManifest = createUiStoryTestbedManifest(),
): readonly UiStoryTestbedPlanItem[] {
  return manifest.families.flatMap((entry) =>
    entry.artifacts.flatMap((artifact): readonly UiStoryTestbedPlanItem[] => {
      if (options.surface != null && artifact.surface !== options.surface) return [];

      return [
        {
          canonicalName: entry.canonicalName,
          title: entry.title,
          packageSubpath: entry.packageSubpath,
          surface: artifact.surface,
          runner: runnerForSurface(artifact.surface),
          status: artifact.status,
          files: artifact.files,
          targetFiles: entry.targetFiles,
          matrixDimensions: entry.matrixDimensions,
          presets: entry.presets,
          viewports: entry.viewports,
          browserSuites: entry.browserSuites,
          harnessHooks: entry.harnessHooks,
          screenshotReview: entry.screenshotReview,
        },
      ];
    }),
  );
}
