import { themePresets } from "../families/foundations/theme/theme-constants.ts";
import {
  browserTestFileFor,
  primaryStoryTargetFor,
  storyFileFor,
  vrtTestFileFor,
  vueTestFileFor,
} from "./story-testbed-paths.ts";
import {
  uiStoryMatrixDimensions,
  uiStoryTestbedBrowserSuiteNames,
  uiStoryTestbedHarnessHookNames,
  uiStoryTestbedScreenshotReview,
  uiStoryTestbedSurfaces,
  type UiStoryTestbedAuditOptions,
  type UiStoryTestbedEntry,
  type UiStoryTestbedSurface,
  type UiStoryTestbedViolation,
  type UiStoryTestbedViolationCode,
} from "./story-testbed-contract.ts";
import { uiStoryTestbedInventory } from "./story-testbed-inventory.ts";

export function auditUiStoryTestbedInventory(
  inventory: readonly UiStoryTestbedEntry[] = uiStoryTestbedInventory,
  options: UiStoryTestbedAuditOptions = {},
): readonly UiStoryTestbedViolation[] {
  const violations: UiStoryTestbedViolation[] = [];
  const seenFamilies = new Set<string>();

  for (const entry of inventory) {
    if (seenFamilies.has(entry.canonicalName)) {
      violations.push({
        code: "duplicate-family",
        family: entry.canonicalName,
        message: "family appears more than once in the story-testbed inventory",
      });
    }
    seenFamilies.add(entry.canonicalName);

    const primaryTarget = primaryStoryTargetFor(entry);
    auditColocatedFiles(violations, entry, primaryTarget);
    auditSharedContracts(violations, entry);
    auditArtifacts(violations, entry, options.existingFiles);
    auditSupportingTests(violations, entry, options.existingFiles);
  }

  return violations;
}

export function formatUiStoryTestbedViolations(
  violations: readonly UiStoryTestbedViolation[],
): string {
  return violations
    .map((violation) => `${violation.code}: ${violation.family}: ${violation.message}`)
    .join("\n");
}

function auditColocatedFiles(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
  primaryTarget: `src/${string}`,
): void {
  if (entry.storyFile !== storyFileFor(entry.canonicalName, primaryTarget)) {
    violations.push({
      code: "misplaced-story-file",
      family: entry.canonicalName,
      message: `expected ${storyFileFor(entry.canonicalName, primaryTarget)}, got ${entry.storyFile}`,
    });
  }

  if (entry.vueTestFile !== vueTestFileFor(entry.canonicalName, primaryTarget)) {
    violations.push({
      code: "misplaced-vue-test-file",
      family: entry.canonicalName,
      message: `expected ${vueTestFileFor(entry.canonicalName, primaryTarget)}, got ${entry.vueTestFile}`,
    });
  }

  if (entry.browserTestFile !== browserTestFileFor(entry.canonicalName, primaryTarget)) {
    violations.push({
      code: "misplaced-browser-test-file",
      family: entry.canonicalName,
      message: `expected ${browserTestFileFor(entry.canonicalName, primaryTarget)}, got ${entry.browserTestFile}`,
    });
  }

  if (entry.vrtTestFile !== vrtTestFileFor(entry.canonicalName, primaryTarget)) {
    violations.push({
      code: "misplaced-vrt-test-file",
      family: entry.canonicalName,
      message: `expected ${vrtTestFileFor(entry.canonicalName, primaryTarget)}, got ${entry.vrtTestFile}`,
    });
  }
}

function auditSharedContracts(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
): void {
  pushMissingValues(
    violations,
    entry,
    "missing-matrix-dimension",
    uiStoryMatrixDimensions,
    entry.matrixDimensions,
    "matrix dimension",
  );
  pushMissingValues(
    violations,
    entry,
    "missing-preset",
    themePresets,
    entry.presets,
    "theme preset",
  );
  pushMissingValues(
    violations,
    entry,
    "missing-browser-suite",
    uiStoryTestbedBrowserSuiteNames,
    entry.browserSuites.map((suite) => suite.name),
    "browser suite",
  );
  pushMissingValues(
    violations,
    entry,
    "missing-harness-hook",
    uiStoryTestbedHarnessHookNames,
    entry.harnessHooks.map((hook) => hook.name),
    "harness hook",
  );
  if (!screenshotReviewMatchesSharedContract(entry.screenshotReview)) {
    violations.push({
      code: "missing-screenshot-review",
      family: entry.canonicalName,
      message: "screenshot review workflow must use the shared Playwright VRT contract",
    });
  }
}

function screenshotReviewMatchesSharedContract(
  review: UiStoryTestbedEntry["screenshotReview"],
): boolean {
  return (
    review.surface === uiStoryTestbedScreenshotReview.surface &&
    review.workflow === uiStoryTestbedScreenshotReview.workflow &&
    review.artifactDirectory === uiStoryTestbedScreenshotReview.artifactDirectory &&
    review.requiresApproval === uiStoryTestbedScreenshotReview.requiresApproval
  );
}

function pushMissingValues<Value extends string>(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
  code: Extract<
    UiStoryTestbedViolationCode,
    | "missing-browser-suite"
    | "missing-harness-hook"
    | "missing-matrix-dimension"
    | "missing-preset"
    | "missing-surface"
  >,
  required: readonly Value[],
  actual: readonly Value[],
  label: string,
): void {
  const actualSet = new Set(actual);
  for (const value of required) {
    if (actualSet.has(value)) continue;
    violations.push({
      code,
      family: entry.canonicalName,
      message: `missing ${label} ${value}`,
    });
  }
}

function auditSupportingTests(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
  existingFiles: ReadonlySet<string> | undefined,
): void {
  if (!existingFiles) return;
  for (const file of entry.supportingTestFiles) {
    if (existingFiles.has(file)) continue;
    violations.push({
      code: "supporting-test-missing",
      family: entry.canonicalName,
      message: `catalogued behavior test ${file} does not exist`,
    });
  }
}

function auditArtifacts(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
  existingFiles: ReadonlySet<string> | undefined,
): void {
  const surfaces = entry.artifacts.map((artifact) => artifact.surface);
  pushMissingValues(
    violations,
    entry,
    "missing-surface",
    uiStoryTestbedSurfaces,
    surfaces,
    "surface",
  );

  const seenSurfaces = new Set<UiStoryTestbedSurface>();
  for (const artifact of entry.artifacts) {
    if (seenSurfaces.has(artifact.surface)) {
      violations.push({
        code: "duplicate-surface",
        family: entry.canonicalName,
        message: `surface ${artifact.surface} appears more than once`,
      });
    }
    seenSurfaces.add(artifact.surface);

    if (artifact.status === "ready") {
      const canonicalFile = canonicalFileForSurface(entry, artifact.surface);
      if (artifact.files.length === 0) {
        violations.push({
          code: "ready-artifact-missing",
          family: entry.canonicalName,
          message: `${artifact.surface} is ready but does not list evidence files`,
        });
      } else if (!artifact.files.includes(canonicalFile)) {
        violations.push({
          code: "ready-artifact-missing",
          family: entry.canonicalName,
          message: `${artifact.surface} ready artifact must list ${canonicalFile}`,
        });
      }
    }

    if (!existingFiles) continue;
    for (const file of artifact.files) {
      const exists = existingFiles.has(file);
      if (artifact.status === "ready" && !exists) {
        violations.push({
          code: "ready-artifact-missing",
          family: entry.canonicalName,
          message: `${artifact.surface} marks ${file} ready but the file does not exist`,
        });
      }
      if (artifact.status === "planned" && exists) {
        violations.push({
          code: "planned-artifact-present",
          family: entry.canonicalName,
          message: `${artifact.surface} file ${file} exists but is still marked planned`,
        });
      }
    }
  }
}

function canonicalFileForSurface(
  entry: UiStoryTestbedEntry,
  surface: UiStoryTestbedSurface,
): `src/${string}` {
  switch (surface) {
    case "musea-story":
      return entry.storyFile;
    case "vue-test-utils":
      return entry.vueTestFile;
    case "vitest-browser":
      return entry.browserTestFile;
    case "playwright-vrt":
      return entry.vrtTestFile;
  }
}
