// The `vize:contracts@0.1.2` WIT types in the component-model JavaScript
// mapping (records as camelCase interfaces, enums as kebab-case string unions,
// variants as `{ tag, val }`, `option<T>` as `T | undefined`, `u32` as
// `number`). `tests/tooling/davinci-extension-sdk.test.ts` pins every
// interface below to the released surface in `crates/vize_extension_sdk/versions/`.

// interface types

export interface Span {
  start: number;
  end: number;
}

export interface Page {
  schemaVersion: number;
  text: string;
}

export type Severity = "error" | "warning" | "info" | "hint";

export type Stage = "source" | "surface" | "semantic" | "lowered" | "emit";

export type PartKind = "primary" | "secondary" | "help" | "suggestion";

export interface DiagnosticPart {
  kind: PartKind;
  span: Span;
  message: string;
}

export type Witness = WitnessLegacyExempt;

export interface WitnessLegacyExempt {
  tag: "legacy-exempt";
  val: string;
}

export interface Diagnostic {
  severity: Severity;
  stage: Stage;
  span: Span;
  message: string;
  parts: Array<DiagnosticPart>;
  witness: Witness | undefined;
}

// interface handshake

export interface Capability {
  protocolVersion: number;
  features: Array<string>;
}

// interface input-lowering

export interface SourceBlock {
  source: string;
  base: number;
  lang: string | undefined;
}

export interface LoweredBlock {
  surface: Page;
  semantic: Page;
  diagnostics: Array<Diagnostic>;
}

// interface expression-analysis

export interface Binding {
  name: string;
  kind: string;
}

export interface Expression {
  id: number;
  source: string;
  span: Span;
}

export interface ExpressionBatch {
  environment: Array<Binding>;
  expressions: Array<Expression>;
}

export interface Analysis {
  facts: Page;
  projection: Page;
  diagnostics: Array<Diagnostic>;
}

// world input-dialect: what a guest exports

export interface Handshake {
  getCapability(): Capability;
}

export interface InputLowering {
  lowerBlock(block: SourceBlock): LoweredBlock;
}

// world expression-dialect: what a guest exports beside `Handshake`

export interface ExpressionAnalysis {
  analyze(batch: ExpressionBatch): Analysis;
}

// world output-target: what a guest exports beside `Handshake`

export interface EmitRequest {
  s2: Page;
  s3: Page;
}

export interface Emitted {
  document: Page;
  diagnostics: Array<Diagnostic>;
}

export interface Emission {
  emit(request: EmitRequest): Emitted;
}

export declare const PACKAGE: "vize:contracts@0.1.2";
export declare const PROTOCOL_VERSION: 1;
export declare const S1_PAGE_SCHEMA: 1;
export declare const S2_PAGE_SCHEMA: 1;
export declare const REQUIRED_FEATURES: readonly ["s1-page@1", "s2-page@1"];
export declare const FACTS_PAGE_SCHEMA: 1;
export declare const PROJECTION_PAGE_SCHEMA: 1;
export declare const EXPRESSION_REQUIRED_FEATURES: readonly ["facts-page@1", "projection-page@1"];
export declare const S3_PAGE_SCHEMA: 1;
export declare const EMIT_DOCUMENT_PAGE_SCHEMA: 1;
export declare const OUTPUT_REQUIRED_FEATURES: readonly [
  "emit-document-page@1",
  "s2-page@1",
  "s3-page@1",
];

/** The capability offer for a guest lowering the given `lang` values. */
export declare function capability(langs: readonly string[]): Capability;
