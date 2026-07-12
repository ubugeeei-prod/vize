/** Built-in lint preset. */
export type LintPreset =
  | "general-recommended"
  | "happy-path"
  | "essential"
  | "incremental"
  | "ecosystem"
  | "opinionated"
  | "nuxt";

/** Options for linting a Vue SFC. */
export interface LintOptions {
  /** Filename reported in the result (default: "anonymous.vue"). */
  filename?: string;
  /** Built-in lint preset (default: "ecosystem"). */
  preset?: LintPreset;
  /** Run only the named rules. */
  enabledRules?: string[];
  /** Locale for diagnostic messages (default: "en"). */
  locale?: "en" | "ja" | "zh";
}

/** One-based source position reported by the linter. */
export interface LintPosition {
  /** One-based line number. */
  line: number;
  /** One-based column number. */
  column: number;
  /** Byte offset in the original source. */
  offset: number;
}

/** Diagnostic reported by the linter. */
export interface LintDiagnostic {
  /** Rule that produced the diagnostic. */
  rule: string;
  /** Diagnostic severity. */
  severity: "error" | "warning";
  /** Localized diagnostic message. */
  message: string;
  /** Location in the original source. */
  location: {
    start: LintPosition;
    end: LintPosition;
  };
  /** Optional remediation hint. */
  help: string | undefined;
}

/** Result of linting a Vue SFC. */
export interface LintResult {
  /** Filename associated with the source. */
  filename: string;
  /** Number of error diagnostics. */
  errorCount: number;
  /** Number of warning diagnostics. */
  warningCount: number;
  /** Diagnostics in source order. */
  diagnostics: LintDiagnostic[];
}

/** Options for formatting a Vue SFC. */
export interface FormatOptions {
  printWidth?: number;
  tabWidth?: number;
  useTabs?: boolean;
  semi?: boolean;
  singleQuote?: boolean;
  jsxSingleQuote?: boolean;
  trailingComma?: "none" | "es5" | "all";
  bracketSpacing?: boolean;
  bracketSameLine?: boolean;
  arrowParens?: "always" | "avoid";
  endOfLine?: "lf" | "crlf" | "cr" | "auto";
  quoteProps?: "as-needed" | "consistent" | "preserve";
  singleAttributePerLine?: boolean;
  vueIndentScriptAndStyle?: boolean;
  sortAttributes?: boolean;
  attributeSortOrder?: "alphabetical" | "as-written";
  mergeBindAndNonBindAttrs?: boolean;
  maxAttributesPerLine?: number | null;
  attributeGroups?: string[][] | null;
  normalizeDirectiveShorthands?: boolean;
  sortBlocks?: boolean;
}

/** Result of formatting a Vue SFC. */
export interface FormatResult {
  /** Formatted source. */
  code: string;
  /** Whether the source changed. */
  changed: boolean;
}
