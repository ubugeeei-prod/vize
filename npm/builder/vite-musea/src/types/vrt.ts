import type { LaunchOptions, Page } from "playwright";

/**
 * Viewport configuration.
 */
export interface ViewportConfig {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Device scale factor */
  deviceScaleFactor?: number;
  /** Viewport name for identification */
  name?: string;
}

/**
 * VRT configuration options.
 */
export interface VrtOptions {
  /** Options forwarded to Playwright's Chromium launcher. */
  launchOptions?: LaunchOptions;
  /** Public gallery route, including the Vite base path. @default '/__musea__' */
  previewBasePath?: string;
  /**
   * Snapshot storage directory.
   * @default '.vize/snapshots'
   */
  snapshotDir?: string;

  /**
   * Pixel difference threshold for comparison.
   * @default 100
   */
  threshold?: number;

  /**
   * Viewports to capture.
   * @default [{ width: 1280, height: 720 }, { width: 375, height: 667 }]
   */
  viewports?: ViewportConfig[];

  /**
   * Number of variants/viewports to capture concurrently.
   * @default 1
   */
  workers?: number;
}

/** Identity of the isolated preview about to be captured. */
export interface BeforeCaptureContext {
  readonly artPath: string;
  readonly variantName: string;
  readonly viewport: Readonly<ViewportConfig>;
}

/** Screenshot capture configuration. */
export interface CaptureConfig {
  /**
   * Run authored Playwright interactions/assertions after preview readiness and
   * waitSelector, before settleTime, hiding, masking and the screenshot.
   * Rejection fails this capture without writing or updating its baseline.
   */
  beforeCapture?: (page: Page, context: BeforeCaptureContext) => void | Promise<void>;
  /** Freeze finite/infinite animations during screenshots. @default "disabled" */
  animations?: "disabled" | "allow";
  /** Emulate prefers-reduced-motion. @default "no-preference" */
  reducedMotion?: "reduce" | "no-preference";
  /** Hide the text caret during screenshots. @default "hide" */
  caret?: "hide" | "initial";
  /** Capture full page vs viewport only */
  fullPage?: boolean;
  /** Wait for Musea async setup and mounted preview. Enabled by the CLI and gallery. */
  waitForPreviewReady?: boolean;
  /** Wait for network idle before capture */
  waitForNetwork?: boolean;
  /** Additional wait time after load (ms) */
  settleTime?: number;
  /** CSS selector to wait for */
  waitSelector?: string;
  /** Elements to hide before capture (CSS selectors) */
  hideElements?: string[];
  /** Elements to cover during capture without changing their layout (CSS selectors). */
  maskElements?: string[];
  /** CSS color used to cover masked elements. @default "#ff00ff" */
  maskColor?: string;
}

/**
 * Image comparison configuration.
 */
export type DiffStyle = "overlay" | "sideBySide" | "diffOnly";

export interface ComparisonConfig {
  /**
   * Per-pixel YIQ color difference threshold from 0 (exact) to 1 (least sensitive).
   * This is separate from `vrt.threshold`, which controls the allowed percentage
   * of pixels that differ.
   * @default 0.1
   */
  colorThreshold?: number;
  /** Anti-aliasing detection */
  antiAliasing?: boolean;
  /** Alpha channel comparison */
  alpha?: boolean;
  /** PNG presentation: overlay (default), baseline/current pair, or transparent unchanged pixels. */
  diffStyle?: DiffStyle;
  /** Diff highlight color */
  diffColor?: { r: number; g: number; b: number };
}

/**
 * CI-specific configuration.
 */
export interface CiConfig {
  /** Fail build on any diff */
  failOnDiff?: boolean;
  /** Auto-update baselines on main branch */
  autoUpdateOnMain?: boolean;
  /** Generate JSON report for CI */
  jsonReport?: boolean;
  /** Retry failed tests */
  retries?: number;
}
