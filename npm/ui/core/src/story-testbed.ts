import { uiFamilyCatalog, type UiFamilyCatalogEntry } from "./family-catalog.ts";
import {
  browserTestFileFor,
  primaryStoryTargetFor,
  storyFileFor,
  vrtTestFileFor,
  vueTestFileFor,
} from "./story-testbed-paths.ts";
import { themePresets } from "./theme-constants.ts";
import type { ThemePresetName } from "./theme-types.ts";

export const UI_STORY_TESTBED_SCHEMA_VERSION = 1;

export const uiStoryTestbedSurfaces = [
  "musea-story",
  "vue-test-utils",
  "vitest-browser",
  "playwright-vrt",
] as const;

export type UiStoryTestbedSurface = (typeof uiStoryTestbedSurfaces)[number];

export type UiStoryTestbedArtifactStatus = "planned" | "ready";

export const uiStoryMatrixDimensions = [
  "states",
  "slots",
  "parts",
  "presets",
  "rtl",
  "reduced-motion",
  "forced-colors",
] as const;

export type UiStoryMatrixDimension = (typeof uiStoryMatrixDimensions)[number];

export interface UiStoryTestbedViewport {
  /** Stable name shared by Musea and Playwright snapshots. */
  readonly name: "desktop" | "mobile";

  /** CSS pixel viewport width. */
  readonly width: number;

  /** CSS pixel viewport height. */
  readonly height: number;

  /** Device scale factor used for visual regression snapshots. */
  readonly deviceScaleFactor: number;
}

export const uiStoryTestbedViewports: readonly UiStoryTestbedViewport[] = [
  { name: "desktop", width: 1280, height: 900, deviceScaleFactor: 1 },
  { name: "mobile", width: 390, height: 844, deviceScaleFactor: 2 },
] as const;

export interface UiStoryTestbedArtifact {
  /** Harness lane owned by this artifact. */
  readonly surface: UiStoryTestbedSurface;

  /** Whether the concrete file is already present or explicitly planned. */
  readonly status: UiStoryTestbedArtifactStatus;

  /** Source-local file paths that prove or will prove the surface. */
  readonly files: readonly `src/${string}`[];
}

export interface UiStoryTestbedEntry {
  /** Stable machine name copied from the public family catalog. */
  readonly canonicalName: string;

  /** Human-readable family title copied from the public family catalog. */
  readonly title: string;

  /** Package subpath this story-testbed entry covers. */
  readonly packageSubpath: UiFamilyCatalogEntry["packageSubpath"];

  /** Primary authored component or composable files that the story drives. */
  readonly targetFiles: readonly `src/${string}`[];

  /** Colocated Musea art file expected for this public family. */
  readonly storyFile: `src/${string}.art.vue`;

  /** Colocated Vue Test Utils story harness spec expected for this public family. */
  readonly vueTestFile: `src/${string}.vue.test.ts`;

  /** Colocated Vitest browser-mode spec expected for this public family. */
  readonly browserTestFile: `src/${string}.browser.spec.ts`;

  /** Colocated Playwright visual-regression spec expected for this public family. */
  readonly vrtTestFile: `src/${string}.vrt.spec.ts`;

  /** Existing catalogued behavior tests that keep the family observable today. */
  readonly supportingTestFiles: readonly `src/${string}.test.ts`[];

  /** Required state/control dimensions for future Musea variants. */
  readonly matrixDimensions: readonly UiStoryMatrixDimension[];

  /** Package theme presets that every visual story matrix must exercise. */
  readonly presets: readonly ThemePresetName[];

  /** Shared viewport set used by browser and VRT lanes. */
  readonly viewports: readonly UiStoryTestbedViewport[];

  /** Current concrete or planned evidence for the issue #4898 lanes. */
  readonly artifacts: readonly UiStoryTestbedArtifact[];
}

export type UiStoryTestbedViolationCode =
  | "duplicate-family"
  | "duplicate-surface"
  | "missing-surface"
  | "missing-matrix-dimension"
  | "missing-preset"
  | "misplaced-story-file"
  | "misplaced-vue-test-file"
  | "misplaced-browser-test-file"
  | "misplaced-vrt-test-file"
  | "supporting-test-missing"
  | "ready-artifact-missing"
  | "planned-artifact-present";

export interface UiStoryTestbedViolation {
  readonly code: UiStoryTestbedViolationCode;
  readonly family: string;
  readonly message: string;
}

export interface UiStoryTestbedAuditOptions {
  /**
   * Source-root file inventory, relative to the package root. When provided,
   * the audit checks ready/planned status against concrete files.
   *
   * @default undefined
   */
  readonly existingFiles?: ReadonlySet<string>;
}

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

function pushMissingValues<Value extends string>(
  violations: UiStoryTestbedViolation[],
  entry: UiStoryTestbedEntry,
  code: Extract<
    UiStoryTestbedViolationCode,
    "missing-matrix-dimension" | "missing-preset" | "missing-surface"
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
