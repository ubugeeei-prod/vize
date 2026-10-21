import type { PluginNode } from "./index.js";
import type { JsonValue } from "./stages.js";

export declare const VISITOR_API_VERSION: 1;
export declare const VISITOR_CAPABILITIES: readonly [
  "sfc",
  "template",
  "script",
  "style",
  "virtual-ts",
  "diagnostics",
  "lint",
  "format",
  "devtools-trace",
];
export type VisitorCapability = (typeof VISITOR_CAPABILITIES)[number];
export type VisitorErrorCode =
  | "invalid-definition"
  | "invalid-name"
  | "invalid-version"
  | "unsupported-api-version"
  | "invalid-capabilities"
  | "unknown-capability"
  | "duplicate-capability"
  | "invalid-visitors"
  | "missing-visitor"
  | "undeclared-visitor"
  | "async-visitor"
  | "invalid-cache-inputs"
  | "invalid-plugin"
  | "invalid-host-contract"
  | "unsupported-capability"
  | "undeclared-capability"
  | "invalid-event"
  | "invalid-visitor-result";

export declare class VisitorPluginValidationError extends TypeError {
  readonly code: VisitorErrorCode;
  readonly field: string;
}

export interface VisitorRange {
  readonly start: number;
  readonly end: number;
  readonly encoding: "utf8";
}
export interface VisitorDiagnostic {
  readonly code: string;
  readonly message: string;
  readonly severity: "error" | "warning" | "information" | "hint";
  readonly range: VisitorRange;
  readonly source?: string;
}
export interface VisitorFix {
  readonly range: VisitorRange;
  readonly text: string;
}
export interface VisitorSfcBlock {
  readonly kind: "template" | "script" | "script-setup" | "style" | "custom";
  readonly language: string | null;
  readonly range: VisitorRange;
  readonly attributes: Readonly<Record<string, string | null>>;
}
export interface VisitorSfcPayload {
  readonly source: string;
  readonly blocks: readonly VisitorSfcBlock[];
}
export interface VisitorTemplatePayload {
  readonly nodes: readonly PluginNode[];
}
export interface VisitorScriptPayload {
  readonly source: string;
  readonly language: "js" | "jsx" | "ts" | "tsx";
  /** Host-owned serialized syntax data. Its schema belongs to the negotiated host. */
  readonly syntax?: JsonValue;
}
export interface VisitorStylePayload {
  readonly source: string;
  readonly language: string;
  readonly scoped: boolean;
  readonly module: boolean | string;
}
export interface VisitorVirtualTsMapping {
  readonly source: VisitorRange;
  readonly generated: VisitorRange;
}
export interface VisitorVirtualTsPayload {
  readonly sourceFile: string;
  readonly virtualFile: string;
  readonly code: string;
  readonly mappings: readonly VisitorVirtualTsMapping[];
}
export interface VisitorDiagnosticsPayload {
  readonly diagnostics: readonly VisitorDiagnostic[];
}
export interface VisitorLintPayload extends VisitorDiagnosticsPayload {
  readonly fixes: readonly VisitorFix[];
}
export interface VisitorFormatPayload {
  readonly source: string;
  readonly formatted: string;
}
export interface VisitorDevtoolsTraceEvent {
  readonly id: string;
  readonly parentId: string | null;
  readonly name: string;
  readonly stage: string;
  readonly startNs: number;
  readonly durationNs: number;
  readonly data?: JsonValue;
}
export interface VisitorDevtoolsTracePayload {
  readonly events: readonly VisitorDevtoolsTraceEvent[];
}
export interface VisitorPayloads {
  readonly sfc: VisitorSfcPayload;
  readonly template: VisitorTemplatePayload;
  readonly script: VisitorScriptPayload;
  readonly style: VisitorStylePayload;
  readonly "virtual-ts": VisitorVirtualTsPayload;
  readonly diagnostics: VisitorDiagnosticsPayload;
  readonly lint: VisitorLintPayload;
  readonly format: VisitorFormatPayload;
  readonly "devtools-trace": VisitorDevtoolsTracePayload;
}
export interface VisitorEvent<C extends VisitorCapability = VisitorCapability> {
  readonly apiVersion: 1;
  readonly capability: C;
  readonly file: string;
  readonly payload: VisitorPayloads[C];
}
export type VisitorHandler<C extends VisitorCapability> = (event: VisitorEvent<C>) => void;
export type VisitorHandlers<C extends readonly VisitorCapability[]> = {
  readonly [K in C[number]]: VisitorHandler<K>;
};
export interface VisitorCacheInput {
  readonly name: string;
  readonly value: string;
}
export interface VisitorPluginDefinition<C extends readonly VisitorCapability[]> {
  readonly apiVersion: 1;
  readonly name: string;
  readonly version: string;
  readonly capabilities: C;
  /** Explicit list of every captured configuration or ambient input. */
  readonly cacheInputs: readonly VisitorCacheInput[];
  readonly visitors: VisitorHandlers<C>;
}
export interface VisitorPlugin<
  C extends readonly VisitorCapability[] = readonly VisitorCapability[],
> {
  readonly kind: "vize.visitor-plugin";
  readonly apiVersion: 1;
  readonly name: string;
  readonly version: string;
  readonly capabilities: C;
  readonly cacheInputs: readonly VisitorCacheInput[];
  readonly visitors: VisitorHandlers<C>;
  readonly fingerprint: string;
}
export interface VisitorHostContract<
  C extends readonly VisitorCapability[] = readonly VisitorCapability[],
> {
  readonly apiVersion: 1;
  readonly capabilities: C;
}
export interface NegotiatedVisitorContract<C extends readonly VisitorCapability[]> {
  readonly apiVersion: 1;
  readonly capabilities: C;
}

/** Define an immutable descriptor. This does not register a runtime hook. */
export declare function defineVisitorPlugin<const C extends readonly VisitorCapability[]>(
  definition: VisitorPluginDefinition<C>,
): VisitorPlugin<C>;
/** Fail closed unless the host provides the exact API version and every declared capability. */
export declare function negotiateVisitorPlugin<const C extends readonly VisitorCapability[]>(
  plugin: VisitorPlugin<C>,
  host: VisitorHostContract,
): NegotiatedVisitorContract<C>;
/** Invoke one visitor in an author fixture; production lifecycle remains host-owned. */
export declare function runVisitorFixture<
  C extends readonly VisitorCapability[],
  K extends C[number],
>(plugin: VisitorPlugin<C>, capability: K, event: VisitorEvent<K>): void;
