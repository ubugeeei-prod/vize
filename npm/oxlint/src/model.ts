import type { OxlintHtmlOptions, OxlintHtmlOutcome } from "@vizejs/native";

export interface PatinaPosition {
  line: number;
  column: number;
  offset: number;
}

export interface PatinaLocation {
  start: PatinaPosition;
  end: PatinaPosition;
}

export interface PatinaDiagnostic {
  rule: string;
  severity: "error" | "warning";
  message: string;
  location: PatinaLocation;
  help: string | null;
  /** Present only when the native caller requests includeFixes. */
  fix?: NativeLintFix | null;
}

export interface NativeLintFix {
  message: string;
  edits: { start: number; end: number; newText: string }[];
}

export interface PatinaLintResult {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: PatinaDiagnostic[];
}

export interface PatinaRuleMeta {
  name: string;
  description: string;
  category: string;
  fixable: boolean;
  defaultSeverity: "error" | "warning";
  presets: PatinaPreset[];
}

/** Existing native parser geometry. Every offset and column is a UTF-8 byte count. */
export interface NativeSfcLocation {
  start: number;
  end: number;
  tagStart: number;
  tagEnd: number;
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
}

export interface NativeSfcBlock {
  content: string;
  loc: NativeSfcLocation;
}

export interface NativeSfcDescriptor {
  source: string;
  template: NativeSfcBlock | null;
  script: (NativeSfcBlock & { setup: boolean }) | null;
  scriptSetup: (NativeSfcBlock & { setup: boolean }) | null;
  styles: NativeSfcBlock[];
  customBlocks: (NativeSfcBlock & { type: string })[];
}

export interface PatinaBinding {
  parseSfc(source: string): NativeSfcDescriptor;
  lintOxlintHtml?: (options: OxlintHtmlOptions) => OxlintHtmlOutcome;
  lintPatinaSfc(
    source: string,
    options?: {
      filename?: string;
      locale?: string;
      helpLevel?: HelpLevel;
      preset?: PatinaPreset;
      enabledRules?: string[];
      includeFixes?: boolean;
      typeAware?: boolean;
      corsaPath?: string;
      componentNameInTemplateCasing?: ComponentNameInTemplateCasingOption;
      customEventNameCasing?: CustomEventNameCasingOption;
      noMutatingProps?: NoMutatingPropsOption;
      defineMacrosOrder?: DefineMacrosOrderOption;
      sfcElementOrder?: SfcElementOrderOption;
      htmlSelfClosing?: HtmlSelfClosingOption;
      vOnEventHyphenation?: HyphenationStyle;
      attributeHyphenation?: HyphenationStyle;
      /** Full CLI-compatible per-rule option map. */
      ruleOptionsJson?: string;
    },
  ): PatinaLintResult;
  getPatinaRules(): PatinaRuleMeta[];
}

export type HelpLevel = "none" | "short" | "full";
export type PatinaPreset =
  | "general-recommended"
  | "essential"
  | "ecosystem"
  | "incremental"
  | "opinionated"
  | "nuxt";

export interface PatinaSettings {
  locale?: string;
  helpLevel?: HelpLevel;
  preset?: PatinaPreset;
  typeAware?: boolean;
  corsaPath?: string;
  /** Rules to evaluate together. Oxlint's own rules still control reporting. */
  rules?: readonly string[] | Record<string, unknown>;
}

export interface PatinaRuleOptions {
  componentNameInTemplateCasing?: ComponentNameInTemplateCasingOption;
  customEventNameCasing?: CustomEventNameCasingOption;
  noMutatingProps?: NoMutatingPropsOption;
  defineMacrosOrder?: DefineMacrosOrderOption;
  sfcElementOrder?: SfcElementOrderOption;
  htmlSelfClosing?: HtmlSelfClosingOption;
  vOnEventHyphenation?: HyphenationStyle;
  attributeHyphenation?: HyphenationStyle;
  /** Full CLI-compatible per-rule option map. */
  ruleOptions?: VizeRuleOptions;
}

export interface VizeRuleOptions {
  "script/define-props-destructuring"?: {
    destructure?: "only-when-assigned" | "always" | "never";
  };
  "html/no-empty-palpable-content"?: { contentDirectives?: string[] };
  "type/strict-boolean-expressions"?: {
    allowString?: boolean;
    allowNumber?: boolean;
    allowNullableObject?: boolean;
    allowNullableBoolean?: boolean;
    allowNullableString?: boolean;
    allowNullableNumber?: boolean;
    allowNullableEnum?: boolean;
    allowAny?: boolean;
  };
  "script/no-restricted-globals"?: {
    globals?: Array<{ name: string; message?: string }>;
  };
  "script/no-restricted-members"?: {
    members?: Array<{ object: string; property: string; message?: string }>;
  };
  "vue/component-name-in-template-casing"?: {
    casing?: ComponentNameInTemplateCasingOption;
    registeredComponentsOnly?: boolean;
    globals?: string[];
  };
  "script/custom-event-name-casing"?: { casing?: CustomEventNameCasingOption };
  "vue/no-mutating-props"?: NoMutatingPropsOption;
  "vue/require-component-registration"?: { globals?: string[] };
  "vue/sfc-element-order"?: SfcElementOrderOption;
  "vue/html-self-closing"?: HtmlSelfClosingOption;
  "vue/html-quotes"?: HtmlQuotesConfig;
  "vue/mustache-interpolation-spacing"?: MustacheInterpolationSpacingStyle;
  "vue/v-on-event-hyphenation"?: HyphenationStyle;
  "vue/v-bind-style"?: VBindStyle;
  "vue/v-on-style"?: VOnStyle;
  "vue/v-slot-style"?: VSlotStyle;
  "vue/attribute-hyphenation"?: HyphenationStyle;
  "vue/valid-v-on"?: { modifiers?: string[] };
  "vue/valid-v-slot"?: { allowModifiers?: boolean };
  "musea/prefer-design-tokens"?: {
    tokens?: Array<{ path: string; value: string; tier?: string }>;
  };
}

export interface LineColumn {
  line: number;
  column: number;
}

export type ComponentNameInTemplateCasingOption = "PascalCase" | "kebab-case";
export type CustomEventNameCasingOption = "camelCase" | "kebab-case";
export interface NoMutatingPropsOption {
  shallowOnly?: boolean;
}
export interface DefineMacrosOrderOption {
  order?: string[];
}
export type SfcElementOrderGroup = string | string[];
export interface SfcElementOrderOption {
  order?: SfcElementOrderGroup[];
}
export type HtmlSelfClosingStyle = "always" | "never" | "any";
export interface HtmlSelfClosingOption {
  html?: {
    void?: HtmlSelfClosingStyle;
    normal?: HtmlSelfClosingStyle;
    component?: HtmlSelfClosingStyle;
  };
  svg?: HtmlSelfClosingStyle;
  math?: HtmlSelfClosingStyle;
}

export type HtmlQuoteStyle = "double" | "single";
export type HtmlQuotesConfig = HtmlQuoteStyle | { style?: HtmlQuoteStyle; avoidEscape?: boolean };
export type MustacheInterpolationSpacingStyle = "always" | "never";
export type HyphenationStyle = "always" | "never";
export type VBindStyle = "shorthand" | "longform";
export type VOnStyle = "shorthand" | "longform";
export type VSlotUniformStyle = "shorthand" | "longform";
export type VSlotStyleValue = VSlotUniformStyle | "v-slot";
export type VSlotStyle =
  | VSlotUniformStyle
  | {
      atComponent?: VSlotStyleValue;
      default?: VSlotStyleValue;
      named?: VSlotUniformStyle;
    };

export type SfcBlockKind = "template" | "script" | "script-setup" | "style" | "custom";

export interface SfcBlock {
  kind: SfcBlockKind;
  name: string;
  content: string;
  contentStart: LineColumn;
  contentEnd: LineColumn;
}

export interface SingleScriptMap {
  block: SfcBlock;
  /**
   * SFC position of the extracted program's first character. Oxlint hands JS
   * plugins the block body without the newline that follows the open tag, so
   * this is not always `block.contentStart`.
   */
  scriptStart: LineColumn;
}
