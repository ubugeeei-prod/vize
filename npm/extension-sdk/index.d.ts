// The `vize:contracts@0.1.0` WIT types in the component-model JavaScript
// mapping (records as camelCase interfaces, enums as kebab-case string unions,
// variants as `{ tag, val }`, `option<T>` as `T | undefined`, `u32` as
// `number`). `tests/tooling/davinci-extension-sdk.test.ts` pins every
// interface below to the released surface in `contracts/versions/`.

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

// world input-dialect: what a guest exports

export interface Handshake {
  getCapability(): Capability;
}

export interface InputLowering {
  lowerBlock(block: SourceBlock): LoweredBlock;
}

export declare const PACKAGE: "vize:contracts@0.1.0";
export declare const PROTOCOL_VERSION: 1;
export declare const S1_PAGE_SCHEMA: 1;
export declare const S2_PAGE_SCHEMA: 1;
export declare const REQUIRED_FEATURES: readonly ["s1-page@1", "s2-page@1"];

/** The capability offer for a guest lowering the given `lang` values. */
export declare function capability(langs: readonly string[]): Capability;
