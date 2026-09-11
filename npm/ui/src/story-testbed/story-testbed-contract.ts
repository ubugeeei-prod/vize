import type { UiFamilyCatalogEntry } from "../catalog/family-catalog.ts";
import type { ThemePresetName } from "../families/foundations/theme/theme-types.ts";

export const UI_STORY_TESTBED_SCHEMA_VERSION = 2;
export const UI_STORY_TESTBED_PACKAGE_NAME = "@vizejs/ui";
export const UI_STORY_TESTBED_SOURCE_ROOT = "npm/ui";

export const uiStoryTestbedSurfaces = [
  "musea-story",
  "vue-test-utils",
  "vitest-browser",
  "playwright-vrt",
] as const;

export type UiStoryTestbedSurface = (typeof uiStoryTestbedSurfaces)[number];

export type UiStoryTestbedArtifactStatus = "planned" | "ready";

export type UiStoryTestbedRunner = "musea" | "vue-test-utils" | "vitest-browser" | "playwright";

export const uiStoryTestbedBrowserSuiteNames = ["focus", "pointer", "layout"] as const;

export type UiStoryTestbedBrowserSuiteName = (typeof uiStoryTestbedBrowserSuiteNames)[number];

export interface UiStoryTestbedBrowserSuite {
  readonly name: UiStoryTestbedBrowserSuiteName;
  readonly surface: Extract<UiStoryTestbedSurface, "vitest-browser">;
  readonly evidence: readonly string[];
}

export const uiStoryTestbedBrowserSuites: readonly UiStoryTestbedBrowserSuite[] = [
  {
    name: "focus",
    surface: "vitest-browser",
    evidence: ["tab order", "roving focus", "focus restoration"],
  },
  {
    name: "pointer",
    surface: "vitest-browser",
    evidence: ["press", "hover", "dismissal"],
  },
  {
    name: "layout",
    surface: "vitest-browser",
    evidence: ["measurement", "viewport constraints", "scroll behavior"],
  },
];

export const uiStoryTestbedHarnessHookNames = [
  "accessibility-tree",
  "live-region-transcript",
  "event-log",
  "bundle-explorer",
  "ssr-hydration-lab",
] as const;

export type UiStoryTestbedHarnessHookName = (typeof uiStoryTestbedHarnessHookNames)[number];

export interface UiStoryTestbedHarnessHook {
  readonly name: UiStoryTestbedHarnessHookName;
  readonly surface: Extract<UiStoryTestbedSurface, "musea-story">;
  readonly output: string;
}

export const uiStoryTestbedHarnessHooks: readonly UiStoryTestbedHarnessHook[] = [
  {
    name: "accessibility-tree",
    surface: "musea-story",
    output: "browser accessibility tree snapshot",
  },
  {
    name: "live-region-transcript",
    surface: "musea-story",
    output: "ordered aria-live announcements",
  },
  {
    name: "event-log",
    surface: "musea-story",
    output: "keyboard, pointer, focus, and emitted component events",
  },
  {
    name: "bundle-explorer",
    surface: "musea-story",
    output: "root and subpath bundle budget summary",
  },
  {
    name: "ssr-hydration-lab",
    surface: "musea-story",
    output: "server markup, hydrated DOM, and hydration diagnostics",
  },
];

export interface UiStoryTestbedScreenshotReview {
  readonly surface: Extract<UiStoryTestbedSurface, "playwright-vrt">;
  readonly workflow: "review-artifacts";
  readonly artifactDirectory: ".vize/artifacts/ui-vrt";
  readonly requiresApproval: true;
}

export const uiStoryTestbedScreenshotReview: UiStoryTestbedScreenshotReview = {
  surface: "playwright-vrt",
  workflow: "review-artifacts",
  artifactDirectory: ".vize/artifacts/ui-vrt",
  requiresApproval: true,
};

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

  /** Browser-mode suites that exercise focus, pointer, and layout behavior. */
  readonly browserSuites: readonly UiStoryTestbedBrowserSuite[];

  /** Musea panels and labs that expose accessibility, event, bundle, and hydration evidence. */
  readonly harnessHooks: readonly UiStoryTestbedHarnessHook[];

  /** Screenshot review contract used by the Playwright VRT lane. */
  readonly screenshotReview: UiStoryTestbedScreenshotReview;

  /** Current concrete or planned evidence for the issue #4898 lanes. */
  readonly artifacts: readonly UiStoryTestbedArtifact[];
}

export type UiStoryTestbedViolationCode =
  | "duplicate-family"
  | "duplicate-surface"
  | "missing-browser-suite"
  | "missing-harness-hook"
  | "missing-screenshot-review"
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

export interface UiStoryTestbedFamilySummary {
  readonly canonicalName: string;
  readonly title: string;
  readonly packageSubpath: UiFamilyCatalogEntry["packageSubpath"];
  readonly targetFileCount: number;
  readonly plannedArtifactCount: number;
  readonly readyArtifactCount: number;
  readonly surfaces: readonly UiStoryTestbedSurface[];
}

export interface UiStoryTestbedManifest {
  readonly schemaVersion: typeof UI_STORY_TESTBED_SCHEMA_VERSION;
  readonly packageName: typeof UI_STORY_TESTBED_PACKAGE_NAME;
  readonly sourceRoot: typeof UI_STORY_TESTBED_SOURCE_ROOT;
  readonly surfaces: readonly UiStoryTestbedSurface[];
  readonly matrixDimensions: readonly UiStoryMatrixDimension[];
  readonly presets: readonly ThemePresetName[];
  readonly viewports: readonly UiStoryTestbedViewport[];
  readonly browserSuites: readonly UiStoryTestbedBrowserSuite[];
  readonly harnessHooks: readonly UiStoryTestbedHarnessHook[];
  readonly screenshotReview: UiStoryTestbedScreenshotReview;
  readonly families: readonly UiStoryTestbedEntry[];
}

export interface UiStoryTestbedPlanItem {
  readonly canonicalName: string;
  readonly title: string;
  readonly packageSubpath: UiFamilyCatalogEntry["packageSubpath"];
  readonly surface: UiStoryTestbedSurface;
  readonly runner: UiStoryTestbedRunner;
  readonly status: UiStoryTestbedArtifactStatus;
  readonly files: readonly `src/${string}`[];
  readonly targetFiles: readonly `src/${string}`[];
  readonly matrixDimensions: readonly UiStoryMatrixDimension[];
  readonly presets: readonly ThemePresetName[];
  readonly viewports: readonly UiStoryTestbedViewport[];
  readonly browserSuites: readonly UiStoryTestbedBrowserSuite[];
  readonly harnessHooks: readonly UiStoryTestbedHarnessHook[];
  readonly screenshotReview: UiStoryTestbedScreenshotReview;
}

export interface UiStoryTestbedPlanOptions {
  readonly surface?: UiStoryTestbedSurface;
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
