/* eslint-disable */

/** Lint options for NAPI */
export interface LintOptionsNapi {
  /** Output format: "text" or "json" */
  format?: string;
  /** Maximum number of warnings before failing */
  maxWarnings?: number;
  /** Quiet mode - only show summary */
  quiet?: boolean;
  /** Automatically fix problems (not yet implemented) */
  fix?: boolean;
  /** Help display level: "full", "short", "none" */
  helpLevel?: string;
  /** Lint preset: "general-recommended", "essential", "incremental", "ecosystem", "opinionated", or "nuxt" */
  preset?: string;
}

/** Lint result for NAPI */
export interface LintResultNapi {
  /** Formatted output string */
  output: string;
  /** Total number of errors */
  errorCount: number;
  /** Total number of warnings */
  warningCount: number;
  /** Number of files linted */
  fileCount: number;
  /** Time in milliseconds */
  timeMs: number;
}

/** Single-file Patina lint options for NAPI */
export interface PatinaLintOptionsNapi {
  /** Filename used for diagnostics */
  filename?: string;
  /** Locale code: "en", "ja", or "zh" */
  locale?: string;
  /** Help display level: "full", "short", or "none" */
  helpLevel?: string;
  /** Lint preset: "general-recommended", "essential", "incremental", "ecosystem", "opinionated", or "nuxt" */
  preset?: string;
  /** Optional list of Patina rule names to enable */
  enabledRules?: Array<string>;
}

/** Get Patina's currently registered rule metadata. */
export declare function getPatinaRules(): any;

/** Lint Vue SFC files matching patterns (native multithreading, .gitignore-aware) */
export declare function lint(
  patterns: Array<string>,
  options?: LintOptionsNapi | undefined | null,
): LintResultNapi;

/** Lint a single Vue SFC with Patina and return structured diagnostics. */
export declare function lintPatinaSfc(
  source: string,
  options?: PatinaLintOptionsNapi | undefined | null,
): any;
