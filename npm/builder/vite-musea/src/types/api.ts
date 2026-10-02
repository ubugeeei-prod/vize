/**
 * Palette API response.
 */
export interface PaletteApiResponse {
  title: string;
  controls: PaletteControl[];
  groups: string[];
  json: string;
  typescript: string;
}

/**
 * Single prop control definition.
 */
export interface PaletteControl {
  name: string;
  control: ControlKind;
  default_value?: unknown;
  description?: string;
  required: boolean;
  options: Array<{ label: string; value: unknown }>;
  range?: { min: number; max: number; step?: number };
  group?: string;
}

/**
 * Supported control kinds for the props panel.
 */
export type ControlKind =
  | "text"
  | "number"
  | "boolean"
  | "range"
  | "select"
  | "radio"
  | "color"
  | "date"
  | "object"
  | "array"
  | "file"
  | "raw";

/**
 * Analysis API response (Props/Emits info).
 */
export interface AnalysisApiResponse {
  props: Array<{
    name: string;
    type: string;
    required: boolean;
    default_value?: unknown;
  }>;
  emits: string[];
}

// ============================================================================
// Accessibility types
// ============================================================================

/**
 * Accessibility testing options.
 */
export interface A11yOptions {
  /** Enable a11y auditing during VRT */
  enabled?: boolean;
  /** axe-core rules to include */
  includeRules?: string[];
  /** axe-core rules to exclude */
  excludeRules?: string[];
  /** WCAG level (A, AA, AAA) */
  level?: "A" | "AA" | "AAA";
}

/**
 * Accessibility audit result.
 */
export interface A11yResult {
  artPath: string;
  variantName: string;
  violations: A11yViolation[];
  passes: number;
  incomplete: number;
  /** Rules requiring manual review, with the same per-node details as violations. */
  incompleteResults?: A11yViolation[];
  /**
   * Why the audit could not run, when it could not run.
   *
   * A variant that failed to audit measured *nothing* about accessibility,
   * so it reports no violations. Recording the failure here instead of as a
   * synthetic `critical` violation keeps the counts honest: a broken run
   * reads as broken rather than as a component with critical defects.
   */
  error?: string;
}

/**
 * Single accessibility violation.
 */
export interface A11yViolation {
  id: string;
  impact: "minor" | "moderate" | "serious" | "critical" | null;
  description: string;
  helpUrl: string;
  nodes: number;
  /** Per-node results from axe; optional for older stored reports. */
  targets?: A11yNode[];
}

export interface A11yNode {
  /** Selectors retain axe's nested arrays for iframe/shadow DOM targets. */
  target: (string | string[])[];
  html: string;
  failureSummary?: string;
  any: A11yCheck[];
  all: A11yCheck[];
  none: A11yCheck[];
}

export interface A11yCheck {
  id: string;
  impact: string;
  message: string;
  /** Measured values such as foreground/background colors and contrast ratios. */
  data?: unknown;
}
