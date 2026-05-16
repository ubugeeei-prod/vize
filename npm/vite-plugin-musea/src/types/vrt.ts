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
}

/**
 * Screenshot capture configuration.
 */
export interface CaptureConfig {
  /** Capture full page vs viewport only */
  fullPage?: boolean;
  /** Wait for network idle before capture */
  waitForNetwork?: boolean;
  /** Additional wait time after load (ms) */
  settleTime?: number;
  /** CSS selector to wait for */
  waitSelector?: string;
  /** Elements to hide before capture (CSS selectors) */
  hideElements?: string[];
  /** Elements to mask before capture (CSS selectors) */
  maskElements?: string[];
}

/**
 * Image comparison configuration.
 */
export interface ComparisonConfig {
  /** Anti-aliasing detection */
  antiAliasing?: boolean;
  /** Alpha channel comparison */
  alpha?: boolean;
  /** Diff image output format */
  diffStyle?: "overlay" | "sideBySide" | "diffOnly" | "animated";
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
