// Feature-specific type definitions (Musea, Patina, Glyph)

// Musea types
export interface ArtParseOptions {
  filename?: string;
}

export interface ArtMetadata {
  title: string;
  description?: string;
  component?: string;
  category?: string;
  tags: string[];
  status: "draft" | "ready" | "deprecated";
  order?: number;
}

export interface ArtVariant {
  name: string;
  template: string;
  isDefault: boolean;
  skipVrt: boolean;
  args?: Record<string, unknown>;
}

export interface ArtStyleBlock {
  content: string;
  scoped: boolean;
}

export interface ArtDescriptor {
  filename: string;
  metadata: ArtMetadata;
  variants: ArtVariant[];
  hasScriptSetup: boolean;
  hasScript: boolean;
  styleCount: number;
  styles: ArtStyleBlock[];
}

export interface CsfOutput {
  code: string;
  filename: string;
}

// Patina (Linter) types
export type LintPreset = "happy-path" | "opinionated" | "essential" | "nuxt";

export interface LintOptions {
  filename?: string;
  /** Built-in lint preset (default: 'happy-path') */
  preset?: LintPreset;
  /** Rules to enable (if not set, all rules are enabled) */
  enabledRules?: string[];
  /** Override severity for specific rules */
  severityOverrides?: Record<string, "error" | "warning" | "off">;
  /** Locale for i18n messages (default: 'en') */
  locale?: "en" | "ja" | "zh";
}

export interface LocaleInfo {
  code: string;
  name: string;
}

export interface LintDiagnostic {
  rule: string;
  severity: "error" | "warning";
  message: string;
  location: {
    start: { line: number; column: number; offset: number };
    end: { line: number; column: number; offset: number };
  };
  help?: string;
}

export interface LintResult {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: LintDiagnostic[];
}

export interface LintRule {
  name: string;
  description: string;
  category: string;
  fixable: boolean;
  defaultSeverity: "error" | "warning";
  presets: LintPreset[];
}

// Glyph (Formatter) types
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

export interface FormatResult {
  code: string;
  changed: boolean;
}
